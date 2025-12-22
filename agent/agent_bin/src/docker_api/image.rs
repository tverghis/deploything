use bollard::{Docker, query_parameters::CreateImageOptionsBuilder};
use tokio_stream::StreamExt;
use tracing::{error, info, instrument};

use crate::docker_api::errors::DockerApiError;

#[derive(Debug)]
pub struct ImageRef {
    name: String,
    tag: String,
}

impl std::fmt::Display for ImageRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.name, self.tag)
    }
}

#[instrument(skip(docker))]
pub async fn pull(docker: &Docker, name: &str, tag: &str) -> Result<ImageRef, DockerApiError> {
    info!("Pulling image");
    let options = CreateImageOptionsBuilder::new()
        .from_image(name)
        .tag(tag)
        .build();

    let create_image_stream = docker.create_image(Some(options), None, None);

    let infos: Result<Vec<_>, _> = create_image_stream.collect().await;

    if let Err(e) = infos {
        error!("Pull failed: {e}");
        return Err(DockerApiError::ImagePullFailed {
            image: name.to_string(),
            tag: tag.to_string(),
        });
    }

    info!("Pull complete");

    Ok(ImageRef {
        name: name.to_string(),
        tag: tag.to_string(),
    })
}
