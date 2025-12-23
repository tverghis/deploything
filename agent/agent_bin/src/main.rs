use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let request = "ws://localhost:4040";
    let (mut stream, _) = tokio_tungstenite::connect_async(request).await.unwrap();

    while let Some(message) = stream.next().await {
        match message {
            Ok(message) => match message {
                Message::Text(bytes) => {
                    let text = bytes.as_str();
                    info!("Got message: {text:?} from the server!");
                }
                Message::Close(_) => {
                    info!("Server closed the connection cleanly.");
                    break;
                }
                _ => unreachable!(),
            },
            Err(e) => {
                error!("Unexpected error in websocket stream: {e}");
                break;
            }
        }
    }
}
