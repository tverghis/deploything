use futures_util::{Sink, SinkExt};
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::{self, Message};
use tracing::error;

use crate::ws::errors::WsError;

pub struct WsSender<S>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    sink: S,
    msg_rx: Receiver<Message>,
}

impl<S> WsSender<S>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    pub fn new(sink: S, msg_rx: Receiver<Message>) -> Self {
        Self { sink, msg_rx }
    }

    pub async fn handle(&mut self) -> Result<(), WsError> {
        while let Some(message) = self.msg_rx.recv().await {
            if let Err(e) = self.sink.send(message).await {
                error!("Failed to send message to control plane: {e}");
            }
        }

        Ok(())
    }
}
