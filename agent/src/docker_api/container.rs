use bollard::{
    Docker, query_parameters::CreateContainerOptionsBuilder, secret::ContainerCreateBody,
};
use tracing::{error, info, instrument};

use crate::docker_api::errors::DockerApiError;

#[instrument(skip(docker))]
pub async fn create(
    docker: &Docker,
    container_name: &str,
    image_name: &str,
) -> Result<(), DockerApiError> {
    info!("Creating container");
    let options = CreateContainerOptionsBuilder::new()
        .name(container_name)
        .build();

    let body = ContainerCreateBody {
        image: Some(image_name.to_string()),
        ..Default::default()
    };

    match docker.create_container(Some(options), body).await {
        Ok(_) => {
            info!("Container create complete");
            Ok(())
        }
        Err(e) => {
            error!("Container create failed: {e}");
            Err(DockerApiError::ContainerCreateFailed {
                image: image_name.to_string(),
            })
        }
    }
}
