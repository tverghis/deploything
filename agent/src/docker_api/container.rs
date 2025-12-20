use bollard::{
    Docker,
    query_parameters::{CreateContainerOptionsBuilder, StartContainerOptions},
    secret::ContainerCreateBody,
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

#[instrument(skip(docker))]
pub async fn start(docker: &Docker, container_name: &str) -> Result<(), DockerApiError> {
    info!("Starting container");

    match docker
        .start_container(container_name, None::<StartContainerOptions>)
        .await
    {
        Ok(_) => {
            info!("Container start complete");
            Ok(())
        }
        Err(e) => {
            error!("Start container failed: {e}");
            Err(DockerApiError::ContainerStartFailed {
                container_name: container_name.to_string(),
            })
        }
    }
}
