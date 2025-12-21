use bollard::Docker;
use tracing::instrument;

use crate::docker_api::errors::DockerApiError;

mod container;
mod errors;
mod image;

pub struct Container<'a> {
    docker: &'a Docker,
    name: String,
}

impl<'a> Container<'a> {
    #[instrument(skip(docker))]
    pub async fn spawn_from_image(
        docker: &'a Docker,
        image_name: &str,
        tag: &str,
    ) -> Result<Self, DockerApiError> {
        let container = Self {
            docker,
            name: format!("{image_name}-{tag}"),
        };

        image::pull(docker, image_name, tag).await?;
        container::create(
            docker,
            &container.name,
            format!("{image_name}:{tag}").as_str(),
        )
        .await?;
        container::start(docker, &container.name).await?;

        Ok(container)
    }

    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<(), DockerApiError> {
        container::stop(self.docker, &self.name).await
    }
}
