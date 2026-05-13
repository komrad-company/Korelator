use std::time::{Duration, UNIX_EPOCH};

use kompiler::RuleLevel;
use serde_json::json;

use crate::Alert;

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
    let alert = Alert::new(
        "rule-001".into(),
        "Test Rule".into(),
        &RuleLevel::Critical,
        json!({"host": "srv"}),
    );
    let json: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&alert).unwrap()).unwrap();

    assert_eq!(json["rule_id"], "rule-001");
    assert_eq!(json["level"], "critical");
    assert!(json["timestamp_unix"].as_u64().unwrap() > 0);
}
