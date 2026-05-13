use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid file format {0}")]
    InvalidFormat(#[from] serde_json::Error),
    #[error("Missing configuration file: {path}")]
    MissingConfigurationFile { path: String },
    #[error("Datasource error: {0}")]
    DatasourceError(#[from] DatasourceError),
    #[error("Database error: {0}")]
    DatabaseError(#[from] konnect::Error),
    #[error("Migration failed: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),
    #[error("Rule parse error: {0}")]
    RuleParseError(#[from] kompiler::Error),
}

#[derive(Debug, Error)]
pub enum DatasourceError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}
