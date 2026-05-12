use std::{env, path::Path};

use khronika::{debug, error, info, intialize_logger};
use kompiler::parse_rules;
use konnect::{Store, init as init_db};
use korelator::{
    AlertSink, AlertStore, DatasourceType, PreparedRule, QuickwitSource, StderrJsonSink,
    StdinSource, load_configuration,
};

#[tokio::main]
async fn main() {
    let config_path =
        env::var("CONFIGURATION_PATH").unwrap_or_else(|_| "configuration.json".to_string());

    let configuration = load_configuration(config_path).unwrap_or_else(|err| {
        eprintln!("Fatal Error: {err}");
        std::process::exit(1)
    });

    intialize_logger(configuration.log);

    let pool = init_db(&configuration.database)
        .await
        .unwrap_or_else(|err| {
            error!("Database connection failed: {err}");
            std::process::exit(1);
        });

    let alert_store = AlertStore::new(pool);
    alert_store.migrate().await.unwrap_or_else(|err| {
        error!("Migration failed: {err}");
        std::process::exit(1);
    });

    let rules_path = Path::new(&configuration.rules_path);
    let parsed = parse_rules(rules_path).unwrap_or_else(|err| {
        error!("Unforgivable error parsing rules: {err}");
        std::process::exit(2);
    });

    let rules: Vec<PreparedRule> = parsed.into_iter().map(PreparedRule::from).collect();
    info!(rules_loaded = rules.len(), "Korelator ready");

    let sink: Box<dyn AlertSink> = Box::new(StderrJsonSink);

    let on_event = |event: serde_json::Value| {
        for rule in &rules {
            if rule.fires_on(&event) {
                debug!(rule_id = rule.id, "rule fired");
                sink.emit(&rule.to_alert(event.clone()));
            }
        }
    };

    match configuration.datasource {
        DatasourceType::Stdin => {
            StdinSource::new()
                .stream(on_event)
                .await
                .unwrap_or_else(|err| {
                    error!("stdin stream error: {err}");
                    std::process::exit(3);
                });
        }
        DatasourceType::Quickwit { url, index } => {
            QuickwitSource::new(url, index)
                .stream(on_event)
                .await
                .unwrap_or_else(|err| {
                    error!("Quickwit stream error: {err}");
                    std::process::exit(3);
                });
        }
    }
}
