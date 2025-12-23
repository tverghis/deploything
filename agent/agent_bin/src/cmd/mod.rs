mod handler;

use agent_wire::deploything::v1::{RemoteCommand, remote_command};
pub use handler::CommandHandler;
use tokio::sync::oneshot;
use tracing::{error, instrument};

#[derive(Debug)]
pub enum CommandResponse {
    ContainerStarted { container_id: String },
    ContainerStopped { container_id: String },
    Error { message: String },
}

#[derive(Debug)]
pub struct CommandBundle {
    inner: RemoteCommand,
    resp_tx: oneshot::Sender<CommandResponse>,
}

impl CommandBundle {
    pub fn new(cmd: RemoteCommand, tx: oneshot::Sender<CommandResponse>) -> Self {
        Self {
            inner: cmd,
            resp_tx: tx,
        }
    }

    pub fn command(&self) -> &remote_command::Command {
        self.inner.command.as_ref().unwrap()
    }

    #[instrument(skip(self))]
    pub fn reply(self, message: CommandResponse) {
        if let Err(_) = self.resp_tx.send(message) {
            error!("Failed to send response");
        }
    }
}
