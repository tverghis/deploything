use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio_tungstenite::tungstenite::{self, Message};

use crate::cmd::CommandBundle;

#[derive(Error, Debug)]
pub enum WsError {
    #[error("Received malformed message: {0}")]
    MessageDecodeError(#[from] prost::DecodeError),

    #[error("Websocket error: {0}")]
    WebsocketError(#[from] tungstenite::Error),

    #[error("Failed to send command to command handler: {0}")]
    CommandSendError(#[from] SendError<CommandBundle>),

    #[error("Failed to send message to websocket sender: {0}")]
    MessageChannelError(#[from] SendError<Message>),
}
