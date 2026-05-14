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
mod tests {
    use super::*;

    use kompiler::RuleLevel;
    use serde_json::json;

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

        let alert = Alert::new(
            "rule-001".into(),
            "Test Rule".into(),
            RuleLevel::High.to_string(),
            json!({"host": "server1"}),
        );
        StderrJsonSink.emit_to(&alert, &mut FailWriter);
    }
}
