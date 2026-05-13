use std::io::{self, Write};

use khronika::warn;

use crate::alert::Alert;

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
