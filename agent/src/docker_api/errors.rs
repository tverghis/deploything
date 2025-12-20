use thiserror::Error;

#[derive(Error, Debug)]
pub enum DockerApiError {
    #[error("failed to pull {image}:{tag}")]
    ImagePulledFailed { image: String, tag: String },
}
