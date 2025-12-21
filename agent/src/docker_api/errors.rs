use thiserror::Error;

#[derive(Error, Debug)]
pub enum DockerApiError {
    #[error("failed to pull {image}:{tag}")]
    ImagePullFailed { image: String, tag: String },
    #[error("failed to create container for image {image}")]
    ContainerCreateFailed { image: String },
    #[error("failed to start container {container_id}")]
    ContainerStartFailed { container_id: String },
    #[error("failed to stop container {container_id}")]
    ContainerStopFailed { container_id: String },
}
