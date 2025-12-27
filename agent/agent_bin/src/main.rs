use std::{sync::Arc, time::Duration};

use agent_bin::{
    cli::AgentCli,
    cmd::CommandHandler,
    docker_api,
    ws::{receiver::WsReceiver, sender::WsSender},
};
use bollard::Docker;
use clap::Parser;
use futures_util::StreamExt;
use prost::Message as _;
use tokio_tungstenite::tungstenite::Message;
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
            snapshot_interval_secs,
        } => {
            run(
                &control_plane_hostname,
                control_plane_port,
                snapshot_interval_secs,
            )
            .await
        }
    };
}

#[instrument]
async fn run(hostname: &str, port: u16, snapshot_interval_secs: u16) {
    let uri = format!("ws://{hostname}:{port}");

    let docker = Docker::connect_with_defaults().unwrap();
    let docker = Arc::new(docker);

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(16);

    let docker1 = docker.clone();
    let cmd_handler = tokio::task::spawn(async move {
        let mut cmd_handler = CommandHandler::new(&docker1, cmd_rx);
        cmd_handler.handle_incoming().await;
    });

    let (stream, _) = tokio_tungstenite::connect_async(&uri).await.unwrap();
    let (sink, stream) = stream.split();
    let (msg_tx, msg_rx) = tokio::sync::mpsc::channel(16);

    let msg_tx2 = msg_tx.clone();
    let ws_receiver = tokio::task::spawn(async move {
        let mut receiver = WsReceiver::new(stream, cmd_tx, msg_tx2);
        receiver.recv().await.unwrap();
    });

    let ws_sender = tokio::task::spawn(async move {
        let mut sender = WsSender::new(sink, msg_rx);
        sender.handle().await.unwrap();
    });

    let snapshot_updater = tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(snapshot_interval_secs as u64)).await;
            let snapshot = docker_api::build_snapshot(&docker)
                .await
                .unwrap()
                .encode_to_vec();
            msg_tx.send(Message::Binary(snapshot.into())).await.unwrap();
        }
    });

    cmd_handler.await.unwrap();
    ws_receiver.await.unwrap();
    ws_sender.await.unwrap();
    snapshot_updater.await.unwrap();
}
