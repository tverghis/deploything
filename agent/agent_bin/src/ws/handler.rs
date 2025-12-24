use futures_util::SinkExt;
use tokio::{
    net::TcpStream,
    sync::{mpsc::Sender, oneshot},
};
use tokio_stream::StreamExt;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
use tracing::instrument;

use crate::{cmd::CommandBundle, ws::errors::WsError};

#[derive(Debug)]
pub struct WsHandler {
    address: String,
    cmd_tx: Sender<CommandBundle>,
}

impl WsHandler {
    pub fn new(address: &str, cmd_tx: Sender<CommandBundle>) -> Self {
        Self {
            address: address.to_string(),
            cmd_tx,
        }
    }

    #[instrument(skip(self))]
    pub async fn connect_and_handle(&mut self) -> Result<(), WsError> {
        let (mut stream, _) = tokio_tungstenite::connect_async(&self.address).await?;

        while let Some(message) = stream.next().await {
            self.handle_message(&mut stream, message?).await?;
        }

        Ok(())
    }

    async fn handle_message(
        &mut self,
        stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
        message: Message,
    ) -> Result<(), WsError> {
        let Message::Binary(bytes) = message else {
            unimplemented!();
        };

        let cmd = prost::Message::decode(bytes)?;
        dbg!(&cmd);

        let (response_tx, response_rx) = oneshot::channel();
        let cmd_bundle = CommandBundle::new(cmd, response_tx);

        self.cmd_tx.send(cmd_bundle).await?;

        let resp = match response_rx.await {
            Ok(_) => "ok".to_string(),
            Err(e) => format!("{e}"),
        };

        Ok(stream.send(Message::Text(resp.into())).await?)
    }
}
