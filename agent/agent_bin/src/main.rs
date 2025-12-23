use agent_bin::cmd::{CommandBundle, CommandHandler, CommandResponse};
use agent_wire::deploything::v1::{RemoteCommand, RunParams, StopParams, remote_command::Command};
use bollard::Docker;
use futures_util::SinkExt;
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let docker = Docker::connect_with_defaults().unwrap();

    let (tx, rx) = tokio::sync::mpsc::channel::<CommandBundle>(16);
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
                            let (response_tx, response_rx) = tokio::sync::oneshot::channel();
                            let command = RemoteCommand {
                                command: Some(Command::Run(RunParams {
                                    image_name: Some(String::from("mccutchen/go-httpbin")),
                                    tag: Some(String::from("latest")),
                                })),
                            };
                            let cmd_bundle = CommandBundle::new(command, response_tx);
                            tx.send(cmd_bundle).await.unwrap();

                            match response_rx.await {
                                Ok(CommandResponse::ContainerStarted { container_id }) => {
                                    info!("Container started successfully: {}", container_id);
                                    stream
                                        .send(Message::Text(container_id.into()))
                                        .await
                                        .unwrap();
                                }
                                Ok(CommandResponse::Error { message }) => {
                                    error!("Failed to start container: {}", message);
                                }
                                Ok(resp) => {
                                    error!("Unexpected response: {:?}", resp);
                                }
                                Err(e) => {
                                    error!("Failed to receive response: {}", e);
                                }
                            }
                        }
                        "stop" => {
                            let (response_tx, response_rx) = tokio::sync::oneshot::channel();
                            let command = RemoteCommand {
                                command: Some(Command::Stop(StopParams {
                                    container_id: Some(String::from("unknown")),
                                })),
                            };
                            let cmd_bundle = CommandBundle::new(command, response_tx);
                            tx.send(cmd_bundle).await.unwrap();

                            match response_rx.await {
                                Ok(CommandResponse::ContainerStopped { container_id }) => {
                                    info!("Container stopped successfully: {}", container_id);
                                }
                                Ok(CommandResponse::Error { message }) => {
                                    error!("Failed to stop container: {}", message);
                                }
                                Ok(resp) => {
                                    error!("Unexpected response: {:?}", resp);
                                }
                                Err(e) => {
                                    error!("Failed to receive response: {}", e);
                                }
                            }
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
