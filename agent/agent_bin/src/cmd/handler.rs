use std::collections::HashMap;

use agent_wire::deploything::v1::remote_command::Command;
use bollard::Docker;
use tokio::sync::mpsc::Receiver;
use tracing::{info, instrument, warn};

use crate::{
    cmd::{CommandBundle, CommandResponse},
    docker_api::{Container, ContainerHostConfig},
};

pub struct CommandHandler<'d> {
    cmd_rx: Receiver<CommandBundle>,
    docker: &'d Docker,
    containers: HashMap<String, Container<'d>>,
}

impl<'d> CommandHandler<'d> {
    pub fn new(docker: &'d Docker, cmd_rx: Receiver<CommandBundle>) -> Self {
        let containers = HashMap::new();
        Self {
            cmd_rx,
            docker,
            containers,
        }
    }

    #[instrument(skip(self))]
    pub async fn handle_incoming(&mut self) {
        while let Some(cmd_bundle) = self.cmd_rx.recv().await {
            let response = match cmd_bundle.command() {
                Command::Run(params) => {
                    info!("Received a RunApplication command from the server");
                    let host_config = ContainerHostConfig::With {
                        port_map: ("8080/tcp", "8080").into(),
                    };
                    match Container::spawn_from_image(
                        &self.docker,
                        &params.image_name(),
                        &params.tag(),
                        host_config,
                    )
                    .await
                    {
                        Ok(container) => {
                            let container_id = container.id().to_string();
                            self.containers.insert(container_id.clone(), container);
                            CommandResponse::ContainerStarted { container_id }
                        }
                        Err(e) => CommandResponse::Error {
                            message: format!("Failed to start container: {}", e),
                        },
                    }
                }
                Command::Stop(params) => {
                    info!(
                        "Received a StopApplication command from the server for container: {}",
                        params.container_id()
                    );

                    match self.containers.get(params.container_id()) {
                        Some(container) => {
                            info!("Stopping container {}", container.id());
                            match container.stop().await {
                                Ok(_) => {
                                    let container_id = params.container_id().to_string();
                                    self.containers.remove(params.container_id());
                                    CommandResponse::ContainerStopped { container_id }
                                }
                                Err(e) => CommandResponse::Error {
                                    message: format!("Failed to stop container: {}", e),
                                },
                            }
                        }
                        None => {
                            warn!(
                                "Received stop command for unknown container {}",
                                params.container_id()
                            );
                            CommandResponse::Error {
                                message: format!("Unknown container: {}", params.container_id()),
                            }
                        }
                    }
                }
            };

            cmd_bundle.reply(response);
        }
    }
}
