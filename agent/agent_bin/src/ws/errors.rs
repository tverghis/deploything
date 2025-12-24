use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio_tungstenite::tungstenite;

use crate::cmd::CommandBundle;

#[derive(Error, Debug)]
pub enum WsError {
    #[error("Received malformed message: {0}")]
    MessageDecodeError(#[from] prost::DecodeError),
    #[error("Websocket error: {0}")]
    WebsocketError(#[from] tungstenite::Error),
    #[error("Failed to send command to handler: {0}")]
    CommandSendError(#[from] SendError<CommandBundle>),
}
