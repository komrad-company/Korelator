use serde::Deserialize;
use serde_json::from_reader;
use std::{fs::File, io::BufReader};

use khronika::configuration::TelemetryConfiguration;
use konnect::DatabaseConfig;

use crate::datasources::DatasourceType;
use crate::errors::Error;

#[derive(Deserialize)]
pub struct Configuration {
    pub datasource: DatasourceType,
    pub rules_path: String,
    pub log: TelemetryConfiguration,
    pub database: DatabaseConfig,
}

pub fn load(path: String) -> Result<Configuration, Error> {
    let file = File::open(&path).map_err(|_| Error::MissingConfigurationFile { path })?;
    let conf = from_reader(BufReader::new(file)).map_err(Error::InvalidFormat)?;
    Ok(conf)
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use super::*;

    fn write_temp_config(name: &str, content: &str) -> std::path::PathBuf {
        let path = env::temp_dir().join(format!("korelator_test_{name}.json"));
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn load_fails_on_missing_file() {
        assert!(matches!(
            load("/tmp/korelator_does_not_exist_xyz.json".into()),
            Err(Error::MissingConfigurationFile { .. })
        ));
    }

    #[test]
    fn load_fails_on_invalid_json() {
        let path = write_temp_config("invalid", "not valid json");
        let result = load(path.to_str().unwrap().into());
        fs::remove_file(path).ok();
        assert!(matches!(result, Err(Error::InvalidFormat(_))));
    }

    #[test]
    fn load_parses_valid_stdin_config() {
        let path = write_temp_config(
            "valid_stdin",
            r#"{
                "datasource": "stdin",
                "rules_path": "/tmp/rules",
                "log": { "level": "error", "file": "/tmp/korelator.log" },
                "database": {
                    "host": "localhost",
                    "port": 5432,
                    "database": "komrad",
                    "user": "korelator",
                    "password": "secret",
                    "schema": "korelator",
                    "search_path": "korelator"
                }
            }"#,
        );
        let result = load(path.to_str().unwrap().into());
        fs::remove_file(path).ok();
        assert!(result.is_ok());
    }
}
