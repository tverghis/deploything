use bollard::{
    Docker,
    models::ContainerCreateBody,
    query_parameters::{
        CreateContainerOptions, StartContainerOptions, StopContainerOptionsBuilder,
    },
    secret::{HostConfig, PortBinding, PortMap},
};
use tracing::{error, info, instrument};

use crate::docker_api::{errors::DockerApiError, image::ImageRef};

#[instrument(skip(docker), ret)]
pub async fn create(
    docker: &Docker,
    image_ref: &ImageRef,
    host_config: ContainerHostConfig,
) -> Result<String, DockerApiError> {
    info!("Creating container");

    let body = ContainerCreateBody {
        image: Some(image_ref.to_string()),
        host_config: host_config.into(),
        ..Default::default()
    };

    match docker
        .create_container(None::<CreateContainerOptions>, body)
        .await
    {
        Ok(res) => {
            info!("Container create complete");
            Ok(res.id)
        }
        Err(e) => {
            error!("Container create failed: {e}");
            Err(DockerApiError::ContainerCreateFailed {
                image: image_ref.to_string(),
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

#[derive(Debug)]
pub enum ContainerHostConfig {
    Empty,
    With { port_map: ContainerPortMap },
}

#[derive(Debug)]
pub struct ContainerPortMap {
    from: String,
    to: String,
}

impl From<(&str, &str)> for ContainerPortMap {
    fn from((src, dst): (&str, &str)) -> Self {
        Self {
            from: src.to_string(),
            to: dst.to_string(),
        }
    }
}

impl From<ContainerPortMap> for PortMap {
    fn from(port_map: ContainerPortMap) -> Self {
        [(
            port_map.from,
            Some(vec![PortBinding {
                host_port: Some(port_map.to),
                ..Default::default()
            }]),
        )]
        .into()
    }
}

impl From<ContainerHostConfig> for Option<HostConfig> {
    fn from(config: ContainerHostConfig) -> Self {
        match config {
            ContainerHostConfig::Empty => None,
            ContainerHostConfig::With { port_map } => Some(HostConfig {
                port_bindings: Some(port_map.into()),
                ..Default::default()
            }),
        }
    }
}
