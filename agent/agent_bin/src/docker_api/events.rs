use std::time::SystemTime;

use bollard::{Docker, query_parameters::EventsOptionsBuilder};
use tokio_stream::StreamExt;
use tracing::{error, info, instrument};

use crate::docker_api::errors::DockerApiError;

pub struct DockerEventsHandler<'a> {
    docker: &'a Docker,
}

impl<'a> DockerEventsHandler<'a> {
    pub fn new(docker: &'a Docker) -> Self {
        Self { docker }
    }

    #[instrument(skip(self))]
    pub async fn listen(&mut self) -> Result<(), DockerApiError> {
        let now = SystemTime::now();
        let epoch_time = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("failed to get current epoch time")
            .as_secs();

        let options = EventsOptionsBuilder::new()
            .since(&epoch_time.to_string())
            .build();
        let mut events_stream = self.docker.events(Some(options));

        info!("Listening for events");

        while let Some(events) = events_stream.next().await {
            match events {
                Ok(events) => {
                    info!("{events:?}");
                }
                Err(e) => {
                    error!("Failed to monitor events: {e}");
                    return Err(DockerApiError::MonitorEventsFailed);
                }
            }
        }

        Ok(())
    }
}
