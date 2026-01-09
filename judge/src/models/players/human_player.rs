use anyhow::Result;
use async_trait::async_trait;
use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

use crate::models::players::Player;
use surrealdb::sql::Thing;

/// HumanPlayer forwards messages from WebSocket to GameLogic and vice versa
/// Forwards messages from GameLogic to WebSocket, forwards messages from WebSocket to GameLogic
pub struct HumanPlayer {
    player_id: Thing,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    move_receiver: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<String>>>,
    is_connected: Arc<Mutex<bool>>,
    timeout_ms: Arc<Mutex<u64>>,
}

impl Clone for HumanPlayer {
    fn clone(&self) -> Self {
        Self {
            player_id: self.player_id.clone(),
            sender: Arc::clone(&self.sender),
            move_receiver: Arc::clone(&self.move_receiver),
            is_connected: Arc::clone(&self.is_connected),
            timeout_ms: Arc::clone(&self.timeout_ms),
        }
    }
}

impl HumanPlayer {
    pub fn new(
        player_id: Thing,
        sender: SplitSink<WebSocket, Message>,
        move_receiver: tokio::sync::mpsc::UnboundedReceiver<String>,
    ) -> Self {
        Self {
            player_id,
            sender: Arc::new(Mutex::new(sender)),
            move_receiver: Arc::new(Mutex::new(move_receiver)),
            is_connected: Arc::new(Mutex::new(true)),
            timeout_ms: Arc::new(Mutex::new(30000)), // Default timeout
        }
    }

    pub async fn disconnect(&self) {
        *self.is_connected.lock().await = false;
    }
}

#[async_trait]
impl Player for HumanPlayer {
    fn player_id(&self) -> &Thing {
        &self.player_id
    }

    async fn is_alive(&self) -> bool {
        *self.is_connected.lock().await
    }

    async fn send_message(&self, message: &str) -> Result<()> {
        tracing::debug!("HumanPlayer {}: Sending message: '{}'", self.player_id, message);

        if !self.is_alive().await {
            tracing::warn!("HumanPlayer {}: Attempted to send message to disconnected player", self.player_id);
            return Err(anyhow::anyhow!("Player disconnected"));
        }

        let mut sender = self.sender.lock().await;
        match sender.send(Message::Text(message.to_string().into())).await {
            Ok(_) => {
                // Flush to prevent buffering that blocks the WebSocket receiver
                if let Err(e) = sender.flush().await {
                    tracing::error!("HumanPlayer {}: Failed to flush message: {:?}", self.player_id, e);
                    return Err(anyhow::anyhow!("Failed to flush message to WebSocket: {}", e));
                }
                tracing::debug!("HumanPlayer {}: Successfully sent message", self.player_id);
                Ok(())
            },
            Err(e) => {
                tracing::error!("HumanPlayer {}: Failed to send message: {:?}", self.player_id, e);
                Err(anyhow::anyhow!("Failed to send message to WebSocket: {}", e))
            }
        }
    }

    async fn receive_message(&self) -> Result<String> {
        if !self.is_alive().await {
            tracing::warn!("HumanPlayer {}: Attempted to receive message from disconnected player", self.player_id);
            return Err(anyhow::anyhow!("Player disconnected"));
        }

        let timeout_ms = *self.timeout_ms.lock().await;
        tracing::debug!("HumanPlayer {}: Waiting for message (timeout: {}ms)", self.player_id, timeout_ms);
        tracing::debug!("HumanPlayer {}: Player connection status: alive={}", self.player_id, self.is_alive().await);
        
        let mut receiver = self.move_receiver.lock().await;
        match timeout(Duration::from_millis(timeout_ms), receiver.recv()).await {
            Ok(Some(move_str)) => {
                tracing::debug!("HumanPlayer {}: Received message: '{}'", self.player_id, move_str);
                Ok(move_str)
            },
            Ok(None) => {
                tracing::error!("HumanPlayer {}: Move channel closed", self.player_id);
                self.disconnect().await;
                Err(anyhow::anyhow!("Move channel closed"))
            }
            Err(_) => {
                tracing::error!("HumanPlayer {}: Move timeout after {}ms - no message received on channel", self.player_id, timeout_ms);
                Err(anyhow::anyhow!("Move timeout"))
            }
        }
    }

    fn set_timeout(&mut self, timeout_ms: u64) {
        // For HumanPlayer, we need to handle the Arc<Mutex<>> wrapper
        // This is a bit tricky since we can't easily mutate through Arc
        // We'll update the timeout through the Arc<Mutex<>>
        let timeout_arc = self.timeout_ms.clone();
        tokio::spawn(async move {
            *timeout_arc.lock().await = timeout_ms;
        });
    }
}

// Implement Player for Arc<HumanPlayer> to allow using Arc<HumanPlayer> as Player
#[async_trait]
impl Player for Arc<HumanPlayer> {
    fn player_id(&self) -> &Thing {
        self.as_ref().player_id()
    }

    async fn is_alive(&self) -> bool {
        self.as_ref().is_alive().await
    }

    async fn send_message(&self, message: &str) -> Result<()> {
        self.as_ref().send_message(message).await
    }

    async fn receive_message(&self) -> Result<String> {
        self.as_ref().receive_message().await
    }

    fn set_timeout(&mut self, timeout_ms: u64) {
        let timeout_arc = self.timeout_ms.clone();
        tokio::spawn(async move {
            *timeout_arc.lock().await = timeout_ms;
        });
    }
}
