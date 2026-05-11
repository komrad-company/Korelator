use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid file format {0}")]
    InvalidFormat(#[from] serde_json::Error),
    #[error("Missing configuration file: {path}")]
    MissingConfigurationFile { path: String },
    #[error("Datasource error: {0}")]
    Datasource(#[from] DatasourceError),
}

#[derive(Debug, Error)]
pub enum DatasourceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
