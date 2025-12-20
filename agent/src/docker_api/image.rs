use bollard::{Docker, query_parameters::CreateImageOptionsBuilder};
use tokio_stream::StreamExt;

use crate::docker_api::errors::DockerApiError;

pub async fn pull(docker: &Docker, name: &str, tag: &str) -> Result<(), DockerApiError> {
    let options = CreateImageOptionsBuilder::new()
        .from_image(name)
        .tag(tag)
        .build();

    let create_image_stream = docker.create_image(Some(options), None, None);

    let infos: Result<Vec<_>, _> = create_image_stream.collect().await;

    if let Err(e) = infos {
        eprintln!("error: {e}");
        return Err(DockerApiError::ImagePulledFailed {
            image: name.to_string(),
            tag: tag.to_string(),
        });
    }

    Ok(())
}
