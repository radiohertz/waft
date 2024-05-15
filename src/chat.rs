//! Contains the handler for the websocket chat endpoint

use std::sync::Arc;

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename = "snake_case")]
pub struct Message {
    /// Type of the message
    r#type: MessageType,
    /// username of the user
    username: String,
    /// Content of the message, only Some if Type is Text
    content: Option<String>,
    // / timestamp of the message
    // ts: DateTime<Utc>,
}

impl Message {
    pub fn new(r#type: MessageType, username: String, content: Option<String>) -> Self {
        Self {
            r#type,
            username,
            content,
        }
    }
}

impl From<Message> for axum::extract::ws::Message {
    fn from(value: Message) -> Self {
        Self::Text(serde_json::to_string(&value).expect("No way this panics lmao"))
    }
}

#[derive(Debug)]
pub struct WsMessageToChatMessageError(String);

impl std::fmt::Display for WsMessageToChatMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Parse Error: {:?}", self.0))
    }
}

impl std::error::Error for WsMessageToChatMessageError {}

impl TryFrom<axum::extract::ws::Message> for Message {
    type Error = WsMessageToChatMessageError;

    fn try_from(value: axum::extract::ws::Message) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value.into_data())
            .map_err(|err| WsMessageToChatMessageError(format!("{err:?}")))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename = "snake_case")]
pub enum MessageType {
    /// Emitted when a user joins the chat
    Join,
    /// Emitted when a user leaves the chat
    Leave,
    /// Emitted when a user sends a text
    Text,
    /// Change the username
    SetUsername,
    /// Sent when username is taken
    UsernameTaken,
}

pub async fn chat_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| chat_handler(socket, state))
}

async fn chat_handler(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut recv) = stream.split();

    let mut username = String::new();
    // First message will always be the Join message
    if let Some(Ok(msg)) = recv.next().await {
        match Message::try_from(msg) {
            Ok(msg) => {
                let mut user_list = state.users.lock().await;
                if msg.username.is_empty() {
                    return;
                }
                if !user_list.contains(&msg.username) {
                    user_list.insert(msg.username.to_string());
                    username = msg.username;
                } else {
                    _ = sender
                        .send(Message::new(MessageType::UsernameTaken, msg.username, None).into())
                        .await;
                    return;
                }
            }
            Err(e) => {
                tracing::info!("Failed to convert message: {e}");
                return;
            }
        }
    }

    tracing::info!("Connected: {username}");

    // send the chat history
    let history: Vec<_> = {
        let history = state._history.read().await;
        history.iter().cloned().collect()
    };
    for msg in history {
        if let Err(e) = sender.send(msg.clone().into()).await {
            tracing::info!("Failed to send history: {e}");
        }
    }
    let mut rx = state.tx.subscribe();

    let uname = username.to_string();
    let mut t2 = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            tracing::trace!("Recv message from other websockets {uname:?}: {msg:?}");
            if let Err(e) = sender.send(msg.into()).await {
                tracing::info!("Error: {e:?}");
                break;
            }
        }
    });

    let tx = state.tx.clone();

    _ = tx.send(Message {
        r#type: MessageType::Join,
        username: username.to_string(),
        content: None,
    });

    let mut t1 = tokio::spawn(async move {
        while let Some(Ok(msg)) = recv.next().await {
            if let Ok(msg) = Message::try_from(msg) {
                tracing::trace!("Recv message from ws: {msg:?}");
                _ = tx.send(msg);
            }
        }
    });

    tokio::select! {
        _ = &mut t1 => t1.abort(),
        _ = &mut t2 => t2.abort(),
    };

    _ = state
        .tx
        .send(Message::new(MessageType::Leave, username.to_string(), None));

    state.users.lock().await.remove(&username);
}
