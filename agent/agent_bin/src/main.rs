use agent_bin::{cli::AgentCli, cmd::CommandHandler, ws::receiver::WsReceiver};
use bollard::Docker;
use clap::Parser;
use futures_util::StreamExt;
use tracing::instrument;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let cli = AgentCli::parse();

    match cli.command {
        agent_bin::cli::Commands::Start {
            control_plane_hostname,
            control_plane_port,
        } => run(&control_plane_hostname, control_plane_port).await,
    };
}

#[instrument]
async fn run(hostname: &str, port: u16) {
    let uri = format!("ws://{hostname}:{port}");

    let docker = Docker::connect_with_defaults().unwrap();

    let (tx, rx) = tokio::sync::mpsc::channel(16);

    let cmd_handler = tokio::task::spawn(async move {
        let mut cmd_handler = CommandHandler::new(&docker, rx);
        cmd_handler.handle_incoming().await;
    });

    let (stream, _) = tokio_tungstenite::connect_async(&uri).await.unwrap();
    let (_sink, stream) = stream.split();

    let ws_handler = tokio::task::spawn(async move {
        let mut ws_handler = WsReceiver::new(stream, tx);
        ws_handler.recv().await.unwrap();
    });

    cmd_handler.await.unwrap();
    ws_handler.await.unwrap();
}
