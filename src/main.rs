use std::io::{self, BufRead};
use std::path::Path;

use serde_json::Value;

use khronika::{debug, error, info, intialize_logger, warn};
use kompiler::parse_rules;

use korelator::{AlertSink, PreparedRule, StderrJsonSink, load_configuration};

fn main() {
    let configuration = load_configuration().unwrap_or_else(|err| {
        eprintln!("Fatal Error: {err}");
        std::process::exit(1)
    });

    intialize_logger(configuration.log);

    let rules_path = Path::new(&configuration.rules_path);
    let parsed = parse_rules(rules_path).unwrap_or_else(|err| {
        error!("Unforgivable error parsing rules: {err}");
        std::process::exit(2);
    });

    let rules: Vec<PreparedRule> = parsed.into_iter().map(PreparedRule::from).collect();
    info!(rules_loaded = rules.len(), "Korelator ready");

    let sink: Box<dyn AlertSink> = Box::new(StderrJsonSink);
    run_event_loop(&rules, sink.as_ref());
}

fn run_event_loop(rules: &[PreparedRule], sink: &dyn AlertSink) {
    let stdin = io::stdin().lock();
    for line in stdin.lines() {
        let Ok(line) = line.inspect_err(|e| error!("stdin read error: {e}")) else {
            break;
        };
        if line.trim().is_empty() {
            continue;
        }

        let Ok(event) = serde_json::from_str::<Value>(&line)
            .inspect_err(|e| warn!("invalid JSON event skipped: {e}"))
        else {
            continue;
        };

        for rule in rules {
            if rule.fires_on(&event) {
                debug!(rule_id = rule.id, "rule fired");
                sink.emit(&rule.to_alert(event.clone()));
            }
        }
    }
}
