use futures_util::{Stream, StreamExt};
use tokio::sync::{mpsc::Sender, oneshot};
use tokio_tungstenite::tungstenite::{self, Message};
use tracing::instrument;

use crate::{cmd::CommandBundle, ws::errors::WsError};

type StreamItem = Result<Message, tungstenite::Error>;

#[derive(Debug)]
pub struct WsReceiver<S>
where
    S: Stream<Item = StreamItem> + Unpin,
{
    stream: S,
    cmd_tx: Sender<CommandBundle>,
}

impl<S> WsReceiver<S>
where
    S: Stream<Item = StreamItem> + Unpin,
{
    pub fn new(stream: S, cmd_tx: Sender<CommandBundle>) -> Self {
        Self { stream, cmd_tx }
    }

    #[instrument(skip(self))]
    pub async fn recv(&mut self) -> Result<(), WsError> {
        while let Some(message) = self.stream.next().await {
            self.handle_message(message?).await?;
        }

        Ok(())
    }

    async fn handle_message(&mut self, message: Message) -> Result<(), WsError> {
        // if message.is_ping() {
        //     return Ok(self.stream.send(Message::Pong(message.into_data())).await?);
        // }

        // TODO: we should probably handle a Close frame.

        let Message::Binary(bytes) = message else {
            unimplemented!();
        };

        let cmd = prost::Message::decode(bytes)?;

        let (response_tx, response_rx) = oneshot::channel();
        let cmd_bundle = CommandBundle::new(cmd, response_tx);

        self.cmd_tx.send(cmd_bundle).await?;

        let _ = match response_rx.await {
            Ok(_) => "ok".to_string(),
            Err(e) => format!("{e}"),
        };

        Ok(())

        // Ok(stream.send(Message::Text(resp.into())).await?)
    }
}
