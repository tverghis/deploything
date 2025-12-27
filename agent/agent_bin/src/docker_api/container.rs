use std::collections::HashMap;

use agent_wire::deploything::v1::{ContainerHostConfig, ContainerStatus};
use bollard::{
    Docker,
    models::ContainerCreateBody,
    query_parameters::{
        CreateContainerOptions, ListContainersOptionsBuilder, StartContainerOptions,
        StopContainerOptionsBuilder,
    },
    secret::{HostConfig, PortBinding},
};
use tracing::{error, info, instrument};

use crate::docker_api::{errors::DockerApiError, image::ImageRef};

#[instrument(skip(docker), ret)]
pub async fn create(
    docker: &Docker,
    image_ref: &ImageRef,
    host_config: Option<&ContainerHostConfig>,
) -> Result<String, DockerApiError> {
    info!("Creating container");

    let host_config = create_host_config(host_config);

    let body = ContainerCreateBody {
        image: Some(image_ref.to_string()),
        host_config,
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

fn create_host_config(from: Option<&ContainerHostConfig>) -> Option<HostConfig> {
    let Some(from) = from else {
        return None;
    };

    let mut port_map = HashMap::new();

    if let Some(pm) = &from.port_map {
        let from_port = pm.from.as_ref().cloned().unwrap();
        let to_port = pm.to.as_ref().cloned().unwrap();

        port_map.insert(
            from_port,
            Some(vec![PortBinding {
                host_port: Some(to_port),
                ..Default::default()
            }]),
        );
    };

    let host_config = HostConfig {
        port_bindings: Some(port_map),
        ..Default::default()
    };

    Some(host_config)
}

#[instrument(skip(docker))]
pub async fn list(docker: &Docker) -> Result<Vec<ContainerStatus>, DockerApiError> {
    let options = ListContainersOptionsBuilder::new().all(true).build();

    match docker.list_containers(Some(options)).await {
        Ok(containers) => {
            info!("List containers complete");
            let containers = containers.iter().map(|c| c.into()).collect();
            Ok(containers)
        }
        Err(e) => {
            error!("List containers failed: {e}");
            Err(DockerApiError::ListContainersFailed)
        }
    }
}
