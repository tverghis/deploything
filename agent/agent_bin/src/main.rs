use agent_bin::{cmd::CommandHandler, ws::handler::WsHandler};
use bollard::Docker;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let docker = Docker::connect_with_defaults().unwrap();

    let (tx, rx) = tokio::sync::mpsc::channel(16);

    let cmd_handler = tokio::task::spawn(async move {
        let mut cmd_handler = CommandHandler::new(&docker, rx);
        cmd_handler.handle_incoming().await;
    });

    let ws_handler = tokio::task::spawn(async move {
        let mut ws_handler = WsHandler::new("ws://localhost:4040", tx);
        ws_handler.connect_and_handle().await.unwrap();
    });

    cmd_handler.await.unwrap();
    ws_handler.await.unwrap();
}
