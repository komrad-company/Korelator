use serde::Deserialize;

use khronika::configuration::TelemetryConfiguration;

#[derive(Deserialize)]
pub struct Configuration {
    pub quickwit_url: String,
    pub rules_path: String,
    pub log: TelemetryConfiguration,
}
