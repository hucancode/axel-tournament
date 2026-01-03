use anyhow::{Result, Context};
use async_trait::async_trait;
use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

use crate::player::Player;

/// HumanPlayer forwards messages from WebSocket to GameLogic and vice versa
/// Forwards messages from GameLogic to WebSocket, forwards messages from WebSocket to GameLogic
pub struct HumanPlayer {
    player_id: String,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    move_receiver: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<String>>>,
    is_connected: Arc<Mutex<bool>>,
    timeout_ms: Arc<Mutex<u64>>,
}

impl HumanPlayer {
    pub fn new(
        player_id: String,
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
    fn player_id(&self) -> &str {
        &self.player_id
    }

    async fn is_alive(&self) -> bool {
        *self.is_connected.lock().await
    }

    async fn send_message(&self, message: &str) -> Result<()> {
        if !self.is_alive().await {
            return Err(anyhow::anyhow!("Player disconnected"));
        }

        let mut sender = self.sender.lock().await;
        sender.send(Message::Text(message.to_string().into())).await
            .context("Failed to send message to WebSocket")?;
        Ok(())
    }

    async fn receive_message(&self) -> Result<String> {
        if !self.is_alive().await {
            return Err(anyhow::anyhow!("Player disconnected"));
        }

        let timeout_ms = *self.timeout_ms.lock().await;
        let mut receiver = self.move_receiver.lock().await;
        match timeout(Duration::from_millis(timeout_ms), receiver.recv()).await {
            Ok(Some(move_str)) => Ok(move_str),
            Ok(None) => {
                self.disconnect().await;
                Err(anyhow::anyhow!("Move channel closed"))
            }
            Err(_) => Err(anyhow::anyhow!("Move timeout")),
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
    fn player_id(&self) -> &str {
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
