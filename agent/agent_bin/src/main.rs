use agent_bin::cmd::CommandHandler;
use agent_wire::deploything::v1::{RemoteCommand, RunParams, StopParams, remote_command::Command};
use bollard::Docker;
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let docker = Docker::connect_with_defaults().unwrap();

    let (tx, rx) = tokio::sync::mpsc::channel::<RemoteCommand>(16);
    tokio::task::spawn(async move {
        let mut cmd_handler = CommandHandler::new(&docker, rx);
        cmd_handler.handle_incoming().await;
    });

    let request = "ws://localhost:4040";
    let (mut stream, _) = tokio_tungstenite::connect_async(request).await.unwrap();

    while let Some(message) = stream.next().await {
        match message {
            Ok(message) => match message {
                Message::Text(bytes) => {
                    let text = bytes.as_str();
                    info!("Got message: {text:?} from the server!");

                    match text.trim() {
                        "start" => {
                            tx.send(RemoteCommand {
                                command: Some(Command::Run(RunParams {
                                    image_name: Some(String::from("mccutchen/go-httpbin")),
                                    tag: Some(String::from("latest")),
                                })),
                            })
                            .await
                            .unwrap();
                        }
                        "stop" => {
                            tx.send(RemoteCommand {
                                command: Some(Command::Stop(StopParams {
                                    container_id: Some(String::from("unknown")),
                                })),
                            })
                            .await
                            .unwrap();
                        }
                        _ => todo!(),
                    }
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
