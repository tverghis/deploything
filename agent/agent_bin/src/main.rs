use agent_bin::{cli::AgentCli, cmd::CommandHandler, ws::handler::WsHandler};
use bollard::Docker;
use clap::Parser;
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

    let ws_handler = tokio::task::spawn(async move {
        let mut ws_handler = WsHandler::new(&uri, tx);
        ws_handler.connect_and_handle().await.unwrap();
    });

    cmd_handler.await.unwrap();
    ws_handler.await.unwrap();
}
