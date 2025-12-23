use std::collections::HashMap;

use agent_wire::deploything::v1::{RemoteCommand, remote_command::Command};
use bollard::Docker;
use tokio::sync::mpsc::Receiver;
use tracing::{info, instrument, warn};

use crate::docker_api::{Container, ContainerHostConfig};

pub struct CommandHandler<'d> {
    cmd_rx: Receiver<RemoteCommand>,
    docker: &'d Docker,
    containers: HashMap<String, Container<'d>>,
}

impl<'d> CommandHandler<'d> {
    pub fn new(docker: &'d Docker, cmd_rx: Receiver<RemoteCommand>) -> Self {
        let containers = HashMap::new();
        Self {
            cmd_rx,
            docker,
            containers,
        }
    }

    #[instrument(skip(self))]
    pub async fn handle_incoming(&mut self) {
        while let Some(remote_cmd) = self.cmd_rx.recv().await {
            match remote_cmd.command.unwrap() {
                Command::Run(params) => {
                    info!("Received a RunApplication command from the server");
                    let host_config = ContainerHostConfig::With {
                        port_map: ("8080/tcp", "8080").into(),
                    };
                    let container = Container::spawn_from_image(
                        &self.docker,
                        &params.image_name(),
                        &params.tag(),
                        host_config,
                    )
                    .await
                    .unwrap();

                    self.containers
                        .insert(container.id().to_string(), container);
                }
                Command::Stop(params) => {
                    info!(
                        "Received a StopApplication command from the server for container: {}",
                        params.container_id()
                    );

                    match self.containers.get(params.container_id()) {
                        Some(container) => {
                            info!("Stopping container {}", container.id());
                            container.stop().await.unwrap();
                        }
                        None => {
                            warn!(
                                "Received stop command for unknown container {}",
                                params.container_id()
                            );
                        }
                    }
                    self.containers.remove(params.container_id());
                }
            }
        }
    }
}
