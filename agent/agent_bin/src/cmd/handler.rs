use agent_wire::deploything::v1::RunApplication;
use tokio::sync::mpsc::Receiver;

pub struct CommandHandler {
    cmd_rx: Receiver<RunApplication>,
}

impl CommandHandler {
    pub fn new(cmd_rx: Receiver<RunApplication>) -> Self {
        Self { cmd_rx }
    }

    pub async fn handle_incoming(&mut self) {
        while let Some(cmd) = self.cmd_rx.recv().await {
            dbg!(cmd);
        }
    }
}
