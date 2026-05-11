use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid file format {0}")]
    InvalidFormat(#[from] serde_json::Error),
    #[error("Missing configuration file: {path}")]
    MissingConfigurationFile { path: String },
}
