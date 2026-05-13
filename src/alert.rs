use std::io::{self, Write};

use khronika::warn;
use kodeks::Alert;

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
mod test {
    use super::*;

    use kompiler::RuleLevel;
    use serde_json::json;

    fn make_alert(level: &RuleLevel) -> Alert {
        let event = json!({"host": "server1"});
        Alert::new("rule-001".into(), "Test Rule".into(), level, event.clone())
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
