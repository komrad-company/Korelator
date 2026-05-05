use serde_json::from_reader;
use std::{env, fs::File, io::BufReader};

pub mod configuration;
pub mod evaluation_engine;

use kompiler::errors::UnforgivableErrors;

pub fn load_configuration() -> Result<configuration::Configuration, UnforgivableErrors> {
    let configuration_path: String =
        env::var("CONFIGURATION_PATH").unwrap_or_else(|_| "configuration.json".to_string());

    let file = File::open(&configuration_path).map_err(|_| {
        UnforgivableErrors::MissingConfigurationFile {
            path: configuration_path,
        }
    })?;

    let reader = BufReader::new(file);
    let conf = from_reader(reader).map_err(UnforgivableErrors::InvalidFormat)?;

    Ok(conf)
}
