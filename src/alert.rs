use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

use khronika::warn;
use kompiler::RuleLevel;

#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub rule_id: String,
    pub title: String,
    pub level: &'static str,
    pub event: Value,
    pub timestamp_unix: u64,
}

impl Alert {
    pub fn new(rule_id: String, title: String, level: &RuleLevel, event: Value) -> Self {
        let timestamp_unix = match SystemTime::now().duration_since(UNIX_EPOCH) {
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
            level: level_str(level),
            event,
            timestamp_unix,
        }
    }
}

fn level_str(level: &RuleLevel) -> &'static str {
    match level {
        RuleLevel::Informational => "informational",
        RuleLevel::Low => "low",
        RuleLevel::Medium => "medium",
        RuleLevel::High => "high",
        RuleLevel::Critical => "critical",
    }
}

pub trait AlertSink: Send + Sync {
    fn emit(&self, alert: &Alert);
}

pub struct StderrJsonSink;

impl AlertSink for StderrJsonSink {
    fn emit(&self, alert: &Alert) {
        match serde_json::to_string(alert) {
            Ok(json) => {
                let mut stderr = io::stderr().lock();
                if let Err(e) = writeln!(stderr, "{json}") {
                    warn!("failed to write alert to stderr: {e}");
                }
            }
            Err(e) => warn!("failed to serialize alert {}: {e}", alert.rule_id),
        }
    }
}
