use agent_wire::deploything::v1::RunApplication;
use bollard::Docker;
use tokio::sync::mpsc::Receiver;
use tracing::{info, instrument};

use crate::docker_api::{Container, ContainerHostConfig};

pub struct CommandHandler {
    cmd_rx: Receiver<RunApplication>,
    docker: Docker,
}

impl CommandHandler {
    pub fn new(cmd_rx: Receiver<RunApplication>) -> Self {
        let docker = Docker::connect_with_defaults().unwrap();
        Self { cmd_rx, docker }
    }

    #[instrument(skip(self))]
    pub async fn handle_incoming(&mut self) {
        while let Some(cmd) = self.cmd_rx.recv().await {
            info!("Received a RunApplication command from the server");
            let host_config = ContainerHostConfig::With {
                port_map: ("8080/tcp", "8080").into(),
            };
            let _ = Container::spawn_from_image(
                &self.docker,
                &cmd.image_name(),
                &cmd.tag(),
                host_config,
            )
            .await;
        }
    }
}
