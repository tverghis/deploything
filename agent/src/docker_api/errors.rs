use thiserror::Error;

#[derive(Error, Debug)]
pub enum DockerApiError {
    #[error("failed to pull {image}:{tag}")]
    ImagePullFailed { image: String, tag: String },
    #[error("failed to create container for image {image}")]
    ContainerCreateFailed { image: String },
    #[error("failed to start container {container_name}")]
    ContainerStartFailed { container_name: String },
}
