use std::collections::HashMap;

use agent_wire::deploything::v1::{RunParams, StopParams, remote_command::Command};
use bollard::Docker;
use tokio::sync::mpsc::Receiver;
use tracing::{instrument, warn};

use crate::{
    cmd::{CommandBundle, CommandResponse},
    docker_api::Container,
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
                Command::Run(params) => self.handle_run_command(params).await,
                Command::Stop(params) => self.handle_stop_command(params).await,
            };

            cmd_bundle.reply(response);
        }
    }

    #[instrument(skip(self), ret)]
    async fn handle_run_command(&mut self, params: &RunParams) -> CommandResponse {
        let container = Container::spawn_from_image(
            &self.docker,
            &params.image_name(),
            &params.tag(),
            params.container_host_config.as_ref(),
        );

        match container.await {
            Ok(container) => {
                // FIXME: why do we need to allocate so many of the same strings here?
                let container_id = container.id().to_string();
                self.containers.insert(container_id.clone(), container);
                CommandResponse::ContainerStarted { container_id }
            }
            Err(e) => CommandResponse::Error {
                message: format!("Failed to start container: {e}"),
            },
        }
    }

    #[instrument(skip(self), ret)]
    async fn handle_stop_command(&mut self, params: &StopParams) -> CommandResponse {
        let container_id = params.container_id();

        let Some(container) = self.containers.get(container_id) else {
            // TODO: should we issue the stop command anyway?
            // Eg: the agent process might have crashed and lost track of the container ID.
            // If the control plane knows of the container ID, we should probably honor the request.
            warn!("Received stop command for unknown container {container_id}",);
            return CommandResponse::Error {
                message: format!("Unknown container: {container_id}"),
            };
        };

        match container.stop().await {
            Ok(_) => {
                let container_id = params.container_id().to_string();
                self.containers.remove(params.container_id());
                CommandResponse::ContainerStopped { container_id }
            }
            Err(e) => CommandResponse::Error {
                message: format!("Failed to stop container: {e}"),
            },
        }
    }
}
