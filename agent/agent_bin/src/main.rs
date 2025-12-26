use agent_bin::{
    cli::AgentCli,
    cmd::CommandHandler,
    ws::{receiver::WsReceiver, sender::WsSender},
};
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

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(16);

    let cmd_handler = tokio::task::spawn(async move {
        let mut cmd_handler = CommandHandler::new(&docker, cmd_rx);
        cmd_handler.handle_incoming().await;
    });

    let (stream, _) = tokio_tungstenite::connect_async(&uri).await.unwrap();
    let (sink, stream) = stream.split();
    let (msg_tx, msg_rx) = tokio::sync::mpsc::channel(16);

    let ws_receiver = tokio::task::spawn(async move {
        let mut receiver = WsReceiver::new(stream, cmd_tx, msg_tx);
        receiver.recv().await.unwrap();
    });

    let ws_sender = tokio::task::spawn(async move {
        let mut sender = WsSender::new(sink, msg_rx);
        sender.handle().await.unwrap();
    });

    cmd_handler.await.unwrap();
    ws_receiver.await.unwrap();
    ws_sender.await.unwrap();
}
