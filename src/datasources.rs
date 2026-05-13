pub(crate) mod quickwit;
pub(crate) mod stdin;

pub use quickwit::QuickwitSource;
pub use stdin::StdinSource;

use serde::Deserialize;
use serde_json::Value;

use crate::errors::DatasourceError;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatasourceType {
    Stdin,
    Quickwit { url: String, index: String },
}

pub async fn run<F: FnMut(Value)>(
    datasource: DatasourceType,
    on_event: F,
) -> Result<(), DatasourceError> {
    match datasource {
        DatasourceType::Stdin => StdinSource::new().stream(on_event).await,
        DatasourceType::Quickwit { url, index } => {
            QuickwitSource::new(url, index).stream(on_event).await
        }
    }
}

#[cfg(test)]
mod tests;
