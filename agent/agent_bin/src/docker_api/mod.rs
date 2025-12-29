use std::time::SystemTime;

use agent_wire::deploything::v1::{AgentSnapshot, ContainerHostConfig};
use bollard::Docker;
use prost_types::Timestamp;
use tracing::instrument;

use crate::docker_api::errors::DockerApiError;

mod container;
mod errors;
mod events;
mod image;

pub use events::DockerEventsHandler;

pub struct Container<'a> {
    docker: &'a Docker,
    id: String,
}

impl<'a> Container<'a> {
    #[instrument(skip(docker))]
    pub async fn spawn_from_image(
        docker: &'a Docker,
        image_name: &str,
        tag: &str,
        host_config: Option<&ContainerHostConfig>,
    ) -> Result<Self, DockerApiError> {
        let image_ref = image::pull(docker, image_name, tag).await?;

        let id = container::create(docker, &image_ref, host_config).await?;
        container::start(docker, &id).await?;

        let container = Self { docker, id };

        Ok(container)
    }

    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<(), DockerApiError> {
        container::stop(self.docker, &self.id).await
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[instrument(skip(docker))]
pub async fn build_snapshot(docker: &Docker) -> Result<AgentSnapshot, DockerApiError> {
    let container_status = container::list(docker).await?;

    let snapshot = AgentSnapshot {
        container_status,
        timestamp: Some(Timestamp::from(SystemTime::now())),
    };

    Ok(snapshot)
}
