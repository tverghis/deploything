use bollard::{
    Docker,
    query_parameters::{
        CreateContainerOptionsBuilder, StartContainerOptions, StopContainerOptionsBuilder,
    },
    secret::ContainerCreateBody,
};
use tracing::{error, info, instrument};

use crate::docker_api::errors::DockerApiError;

#[instrument(skip(docker), ret)]
pub async fn create(docker: &Docker, image_name: &str) -> Result<String, DockerApiError> {
    info!("Creating container");
    let options = CreateContainerOptionsBuilder::new().build();

    let body = ContainerCreateBody {
        image: Some(image_name.to_string()),
        ..Default::default()
    };

    match docker.create_container(Some(options), body).await {
        Ok(res) => {
            info!("Container create complete");
            Ok(res.id)
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
pub async fn start(docker: &Docker, container_id: &str) -> Result<(), DockerApiError> {
    info!("Starting container");

    match docker
        .start_container(container_id, None::<StartContainerOptions>)
        .await
    {
        Ok(_) => {
            info!("Container start complete");
            Ok(())
        }
        Err(e) => {
            error!("Start container failed: {e}");
            Err(DockerApiError::ContainerStartFailed {
                container_id: container_id.to_string(),
            })
        }
    }
}

#[instrument(skip(docker))]
pub async fn stop(docker: &Docker, container_id: &str) -> Result<(), DockerApiError> {
    info!("Stopping container");

    // TODO: allow configuration of timeout
    let options = StopContainerOptionsBuilder::new().t(10).build();

    match docker.stop_container(container_id, Some(options)).await {
        Ok(_) => {
            info!("Container stopped");
            Ok(())
        }
        Err(e) => {
            error!("Container stop failed: {e}");
            Err(DockerApiError::ContainerStopFailed {
                container_id: container_id.to_string(),
            })
        }
    }
}
