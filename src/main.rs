use std::{env, path::Path};

use khronika::{debug, error, info, initialize_logger};
use korelator::{
    AlertSink, AlertStore, StderrJsonSink, load_configuration, load_rules, run_datasource,
};

#[tokio::main]
async fn main() {
    let config_path =
        env::var("CONFIGURATION_PATH").unwrap_or_else(|_| "configuration.json".to_string());

    let configuration = load_configuration(config_path).unwrap_or_else(|err| {
        eprintln!("Fatal Error: {err}");
        std::process::exit(1)
    });

    let _ = initialize_logger(configuration.log).unwrap_or_else(|err| {
        eprintln!("Fatal: failed to initialize logger: {err}");
        std::process::exit(1);
    });

    let rules = load_rules(Path::new(&configuration.rules_path)).unwrap_or_else(|err| {
        error!("Unforgivable error parsing rules: {err}");
        std::process::exit(2);
    });

    info!(rules_loaded = rules.len(), "Korelator ready");

    let store = AlertStore::setup(&configuration.database)
        .await
        .unwrap_or_else(|err| {
            error!("Startup failed: {err}");
            std::process::exit(1);
        });

    let tx = store.spawn_persist_task();
    let sink: Box<dyn AlertSink> = Box::new(StderrJsonSink);

    let on_event = move |event: serde_json::Value| {
        for rule in &rules {
            if rule.fires_on(&event) {
                debug!(rule_id = rule.id, "rule fired");
                let alert = rule.to_alert(event.clone());
                sink.emit(&alert);
                if tx.send(alert).is_err() {
                    error!(rule_id = rule.id, "persist channel closed, alert dropped");
                }
            }
        }
    };

    run_datasource(configuration.datasource, on_event)
        .await
        .unwrap_or_else(|err| {
            error!("Datasource error: {err}");
            std::process::exit(3);
        });
}
