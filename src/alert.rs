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

pub trait AlertSink: Send + Sync {
    fn emit(&self, alert: &Alert);
}

pub struct StderrJsonSink;

impl AlertSink for StderrJsonSink {
    fn emit(&self, alert: &Alert) {
        self.emit_to(alert, &mut io::stderr().lock());
    }
}

impl StderrJsonSink {
    pub(crate) fn emit_to<W: Write>(&self, alert: &Alert, writer: &mut W) {
        let json = serde_json::to_string(alert).expect("Alert serialization is infallible");
        if let Err(e) = writeln!(writer, "{json}") {
            warn!("failed to write alert to stderr: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::time::Duration;

    use kompiler::RuleLevel;
    use serde_json::json;

    use super::*;

    fn make_alert(level: &RuleLevel) -> Alert {
        Alert::new(
            "rule-001".into(),
            "Test Rule".into(),
            level,
            json!({"host": "srv"}),
        )
    }

    #[test]
    fn alert_new_sets_all_fields() {
        let event = json!({"host": "server1"});
        let alert = Alert::new(
            "rule-001".into(),
            "My Rule".into(),
            &RuleLevel::High,
            event.clone(),
        );

        assert_eq!(alert.rule_id, "rule-001");
        assert_eq!(alert.title, "My Rule");
        assert_eq!(alert.level, "high");
        assert_eq!(alert.event, event);
        assert!(alert.timestamp_unix > 0);
    }

    #[test]
    fn alert_new_at_before_epoch_sets_timestamp_to_zero() {
        let before_epoch = UNIX_EPOCH - Duration::from_secs(100);
        let alert = Alert::new_at(
            "rule-001".into(),
            "Test".into(),
            &RuleLevel::Low,
            json!({}),
            before_epoch,
        );
        assert_eq!(alert.timestamp_unix, 0);
    }

    #[test]
    fn alert_serializes_to_json_with_expected_fields() {
        let alert = make_alert(&RuleLevel::Critical);
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&alert).unwrap()).unwrap();

        assert_eq!(json["rule_id"], "rule-001");
        assert_eq!(json["level"], "critical");
        assert!(json["timestamp_unix"].as_u64().unwrap() > 0);
    }

    #[test]
    fn stderr_json_sink_emits_without_panic() {
        StderrJsonSink.emit(&make_alert(&RuleLevel::High));
    }

    #[test]
    fn emit_to_handles_write_failure() {
        struct FailWriter;
        impl Write for FailWriter {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "test failure"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        StderrJsonSink.emit_to(&make_alert(&RuleLevel::High), &mut FailWriter);
    }
}
