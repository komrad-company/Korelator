use std::collections::HashMap;

use serde_json::json;

use kompiler::RuleLevel;
use kompiler::{
    AggregationType, Condition, FieldFilter, FilterTypes, Filters, Matcher, Rule, Types,
};

use crate::PreparedRule;

fn make_rule(matcher: Matcher) -> Rule {
    Rule {
        id: "rule-test".into(),
        title: "Test rule".into(),
        level: RuleLevel::High,

        description: None,
        tags: None,
        matcher,
        filters: HashMap::from([(
            "process".to_string(),
            vec![Filters(vec![FieldFilter {
                field: "process_name".into(),
                condition: FilterTypes::Contains,
                values: vec![Types::String("shell".into())],
            }])],
        )]),
        condition: Condition::Filter("process".into()),
    }
}

#[test]
fn single_matcher_fires_when_condition_matches() {
    let prepared = PreparedRule::from(make_rule(Matcher::Single));
    assert!(prepared.fires_on(&json!({ "process_name": "bash_shell" })));
}

#[test]
fn single_matcher_does_not_fire_when_condition_misses() {
    let prepared = PreparedRule::from(make_rule(Matcher::Single));
    assert!(!prepared.fires_on(&json!({ "process_name": "nginx" })));
}

#[test]
fn threshold_matcher_is_not_yet_implemented() {
    let prepared = PreparedRule::from(make_rule(Matcher::Threshold {
        timeframe_secs: 60,
        aggregate: AggregationType::Count(10),
        group_by: vec![],
    }));
    // Until Threshold is implemented, it must never fire and must not crash.
    assert!(!prepared.fires_on(&json!({ "process_name": "bash_shell" })));
}

#[test]
fn multi_group_filters_are_flattened_per_name() {
    // Two filter groups under the same name "process" — must be OR-combined.
    let rule = Rule {
        id: "rule-flatten".into(),
        title: "Flatten test".into(),
        level: RuleLevel::Low,
        description: None,
        tags: None,
        matcher: Matcher::Single,
        filters: HashMap::from([(
            "process".to_string(),
            vec![
                Filters(vec![FieldFilter {
                    field: "process_name".into(),
                    condition: FilterTypes::Contains,
                    values: vec![Types::String("shell".into())],
                }]),
                Filters(vec![FieldFilter {
                    field: "pid".into(),
                    condition: FilterTypes::Gt,
                    values: vec![Types::Integer(1000)],
                }]),
            ],
        )]),
        condition: Condition::Filter("process".into()),
    };

    let prepared = PreparedRule::from(rule);

    assert!(prepared.fires_on(&json!({ "process_name": "bash_shell", "pid": 1 })));
    assert!(prepared.fires_on(&json!({ "process_name": "nginx", "pid": 9999 })));
    assert!(!prepared.fires_on(&json!({ "process_name": "nginx", "pid": 1 })));
}

#[test]
fn to_alert_carries_rule_metadata_and_event() {
    let prepared = PreparedRule::from(make_rule(Matcher::Single));
    let event = json!({ "process_name": "bash_shell" });
    let alert = prepared.to_alert(event.clone());

    assert_eq!(alert.rule_id, "rule-test");
    assert_eq!(alert.title, "Test rule");
    assert_eq!(alert.level, RuleLevel::High);
    assert_eq!(alert.event, event);
}
