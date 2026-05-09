// Room WebSocket handler. Spec: judge/protocols/wire.md.

use crate::middleware::auth::validate_jwt;
use crate::protocol::{parse_client, serialize_server, ClientFrame, ParseError, ServerFrame};
use crate::services::storage::Event;
use crate::services::room_logic::{LiveRoom, RoomLogic, RoomRegistry};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;

const LEASE_TTL: Duration = Duration::from_secs(15);
const HELLO_TIMEOUT: Duration = Duration::from_secs(10);
const PING_INTERVAL: Duration = Duration::from_secs(20);

pub struct WsContext<L: RoomLogic> {
    pub registry: Arc<RoomRegistry<L>>,
    pub jwt_secret: String,
}

pub async fn handle_ws<L: RoomLogic>(socket: WebSocket, room_id: String, ctx: Arc<WsContext<L>>) {
    let (mut tx, mut rx) = socket.split();

    // 1. HELLO with timeout.
    let hello = match tokio::time::timeout(HELLO_TIMEOUT, rx.next()).await {
        Ok(Some(Ok(Message::Text(t)))) => t,
        Ok(Some(Ok(Message::Close(_)))) | Ok(None) => return,
        Ok(Some(Err(_))) => return,
        Err(_) => {
            send_err(&mut tx, "TIMEOUT", "hello timeout").await;
            return;
        }
        _ => {
            send_err(&mut tx, "BAD_FRAME", "expected text HELLO").await;
            return;
        }
    };

    let (jwt, since) = match parse_client(&hello) {
        Ok(ClientFrame::Hello { jwt, since }) => (jwt, since),
        Ok(_) => {
            send_err(&mut tx, "EXPECTED_HELLO", "first frame must be HELLO").await;
            return;
        }
        Err(e) => {
            send_err(&mut tx, "PARSE", &format!("{e}")).await;
            return;
        }
    };

    let player_id = match validate_jwt(&jwt, &ctx.jwt_secret) {
        Ok(claims) => claims.sub,
        Err(e) => {
            send_err(&mut tx, "AUTH", &e).await;
            return;
        }
    };

    // 2. Open the room (acquires lease if not already owned).
    let room = match ctx.registry.open(&room_id, LEASE_TTL).await {
        Ok(r) => r,
        Err(e) => {
            send_err(&mut tx, "ROOM_OPEN", &format!("{e}")).await;
            return;
        }
    };

    // 3. Subscribe BEFORE snapshotting head so we don't miss live events.
    let mut subscriber = room.subscribe();
    let head_at_subscribe = room.head();

    // 4. WELCOME.
    if send_frame(
        &mut tx,
        &ServerFrame::Welcome {
            player_id: player_id.clone(),
            head: head_at_subscribe,
        },
    )
    .await
    .is_err()
    {
        return;
    }

    // 5. Gap fill: events the client missed.
    if since < head_at_subscribe {
        match room.read_since(since).await {
            Ok(events) => {
                for e in events {
                    if e.seq > head_at_subscribe {
                        break;
                    }
                    if send_event(&mut tx, &e).await.is_err() {
                        return;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Gap fill read failed: {e}");
                send_err(&mut tx, "READ", "gap fill failed").await;
                return;
            }
        }
    }

    // 6. Writer task (broadcast -> ws) + reader loop (ws -> ACT).
    let (writer_done_tx, mut writer_done_rx) = tokio::sync::oneshot::channel::<()>();
    let writer = tokio::spawn(async move {
        let mut ping_ticker = tokio::time::interval(PING_INTERVAL);
        ping_ticker.tick().await; // skip immediate
        loop {
            tokio::select! {
                msg = subscriber.recv() => {
                    match msg {
                        Ok(event) => {
                            if event.seq <= head_at_subscribe {
                                continue;
                            }
                            if send_event(&mut tx, &event).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("WS subscriber lagged by {n}; closing");
                            break;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                _ = ping_ticker.tick() => {
                    if send_frame(&mut tx, &ServerFrame::Ping).await.is_err() {
                        break;
                    }
                }
            }
        }
        let _ = writer_done_tx.send(());
    });

    let room_for_reader: Arc<LiveRoom<L>> = room.clone();
    loop {
        tokio::select! {
            _ = &mut writer_done_rx => break,
            msg = rx.next() => {
                let Some(msg) = msg else { break };
                let text = match msg {
                    Ok(Message::Text(t)) => t,
                    Ok(Message::Close(_)) | Err(_) => break,
                    _ => continue,
                };
                let frame = match parse_client(&text) {
                    Ok(f) => f,
                    Err(ParseError::Empty) => continue,
                    Err(e) => {
                        tracing::debug!("Bad client frame from {player_id}: {e}");
                        continue;
                    }
                };
                match frame {
                    ClientFrame::Pong => continue,
                    ClientFrame::Hello { .. } => {
                        tracing::debug!("Spurious HELLO from {player_id}; ignoring");
                        continue;
                    }
                    ClientFrame::Act { kind, payload } => {
                        if let Err(e) = room_for_reader
                            .handle_act(&player_id, &kind, &payload)
                            .await
                        {
                            tracing::debug!("ACT rejected for {player_id}: {e}");
                        }
                    }
                }
            }
        }
    }

    writer.abort();
}

async fn send_frame(
    tx: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    frame: &ServerFrame,
) -> Result<(), axum::Error> {
    let s = serialize_server(frame);
    tx.send(Message::Text(s.into())).await?;
    tx.flush().await
}

async fn send_event(
    tx: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    event: &Event,
) -> Result<(), axum::Error> {
    send_frame(
        tx,
        &ServerFrame::Event {
            seq: event.seq,
            kind: event.kind.clone(),
            payload: event.payload.clone(),
        },
    )
    .await
}

async fn send_err(
    tx: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    code: &str,
    msg: &str,
) {
    let _ = send_frame(
        tx,
        &ServerFrame::Err {
            code: code.into(),
            msg: msg.into(),
        },
    )
    .await;
}
