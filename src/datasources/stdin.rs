use serde_json::Value;
use tokio::io::{self, AsyncBufRead, AsyncBufReadExt, BufReader};

use crate::errors::DatasourceError;

use khronika::warn;

#[derive(Default)]
pub struct StdinSource;

impl StdinSource {
    pub fn new() -> Self {
        Self
    }

    pub(crate) async fn stream_from<R, F>(reader: R, mut on_event: F) -> Result<(), DatasourceError>
    where
        R: AsyncBufRead + Unpin,
        F: FnMut(Value),
    {
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<Value>(&line) {
                Ok(event) => on_event(event),
                Err(e) => warn!("invalid JSON event skipped: {e}"),
            }
        }
        Ok(())
    }

    pub async fn stream<F>(&self, on_event: F) -> Result<(), DatasourceError>
    where
        F: FnMut(Value),
    {
        Self::stream_from(BufReader::new(io::stdin()), on_event).await
    }
}
