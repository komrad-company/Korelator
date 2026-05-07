use std::path::Path;

use khronika::{debug, error, intialize_logger};
use kompiler::rules::parse_rules;

use korelator::load_configuration;

fn main() {
    let configuration = load_configuration().unwrap_or_else(|err| {
        eprintln!("Fatal Error: {err}");
        std::process::exit(1)
    });

    intialize_logger(configuration.log);
    debug!("Korelator successfully initiated");

    let rules_path = Path::new(&configuration.rules_path);
    let parsed_rules = parse_rules(rules_path)
        .map_err(|e| {
            error!("Unforgivable error {e}");
            std::process::exit(2)
        })
        .unwrap();

    dbg!(parsed_rules.len());
}
