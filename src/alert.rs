use serde::Serialize;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

use khronika::warn;
use kompiler::RuleLevel;

pub(crate) mod db;
pub(crate) mod sink;

pub use db::AlertRow;
pub use sink::{AlertSink, StderrJsonSink};

#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub rule_id: String,
    pub title: String,
    pub level: String,
    pub event: Value,
    pub timestamp_unix: u64,
}

impl Alert {
    pub fn new(rule_id: String, title: String, level: &RuleLevel, event: Value) -> Self {
        Self::new_at(rule_id, title, level, event, SystemTime::now())
    }

    pub(crate) fn new_at(
        rule_id: String,
        title: String,
        level: &RuleLevel,
        event: Value,
        time: SystemTime,
    ) -> Self {
        let timestamp_unix = match time.duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(e) => {
                warn!(
                    rule_id = rule_id,
                    "system clock is before UNIX_EPOCH ({e}), emitting alert with timestamp_unix=0"
                );
                0
            }
        };

        Self {
            rule_id,
            title,
            level: level.to_string(),
            event,
            timestamp_unix,
        }
    }
}

#[cfg(test)]
mod tests;
