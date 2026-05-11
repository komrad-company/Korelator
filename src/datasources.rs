pub mod quickwit;
pub mod stdin;

pub use quickwit::QuickwitSource;
pub use stdin::StdinSource;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatasourceType {
    Stdin,
    Quickwit { url: String, index: String },
}

#[cfg(test)]
mod tests;
