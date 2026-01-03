use std::{sync::Arc, time::Duration};

use agent_bin::{
    cli::AgentCli,
    cmd::CommandHandler,
    docker_api::{self, DockerEventsHandler},
    ws::{receiver::WsReceiver, sender::WsSender},
};
use bollard::Docker;
use clap::Parser;
use futures_util::{StreamExt, future::join_all};
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

    let cmd_handler = {
        let docker = docker.clone();
        tokio::task::spawn(async move {
            let mut cmd_handler = CommandHandler::new(&docker, cmd_rx);
            cmd_handler.handle_incoming().await;
        })
    };

    let events_monitor = {
        let docker = docker.clone();
        tokio::task::spawn(async move {
            let mut events_handler = DockerEventsHandler::new(&docker);
            let _ = events_handler.listen().await;
        })
    };

    let (stream, _) = tokio_tungstenite::connect_async(&uri).await.unwrap();
    let (sink, stream) = stream.split();
    let (msg_tx, msg_rx) = tokio::sync::mpsc::channel(16);

    let ws_receiver = {
        let msg_tx = msg_tx.clone();
        tokio::task::spawn(async move {
            let mut receiver = WsReceiver::new(stream, cmd_tx, msg_tx);
            receiver.recv().await.unwrap();
        })
    };

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

    let tasks = vec![
        cmd_handler,
        events_monitor,
        ws_receiver,
        ws_sender,
        snapshot_updater,
    ];

    join_all(tasks).await;
}
