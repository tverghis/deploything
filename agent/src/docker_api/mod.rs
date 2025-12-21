use bollard::Docker;
use tracing::instrument;

use crate::docker_api::errors::DockerApiError;

mod container;
mod errors;
mod image;

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
    ) -> Result<Self, DockerApiError> {
        image::pull(docker, image_name, tag).await?;

        let image_ref = format!("{image_name}:{tag}");
        let id = container::create(docker, &image_ref).await?;

        container::start(docker, &id).await?;

        let container = Self { docker, id };

        Ok(container)
    }

    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<(), DockerApiError> {
        container::stop(self.docker, &self.id).await
    }
}
