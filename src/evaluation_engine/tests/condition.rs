use serde_json::json;
use std::collections::HashMap;

use kompiler::{Condition, FieldFilter, FilterTypes, Filters, Types};

use crate::evaluation_engine::{Evaluate, EvaluationContext};

fn make_ctx(filters: HashMap<String, Filters>) -> EvaluationContext {
    EvaluationContext::new(filters)
}

#[test]
fn or_matches_if_any_filter_matches() {
    let ctx = make_ctx(HashMap::from([
        (
            "process".into(),
            Filters(vec![FieldFilter {
                field: "process_name".into(),
                condition: FilterTypes::Contains,
                values: vec![Types::String("shell".into())],
            }]),
        ),
        (
            "user".into(),
            Filters(vec![FieldFilter {
                field: "username".into(),
                condition: FilterTypes::Contains,
                values: vec![Types::String("admin".into())],
            }]),
        ),
    ]));

    let condition = Condition::Or(
        Box::new(Condition::Filter("process".into())),
        Box::new(Condition::Filter("user".into())),
    );

    assert!(condition.evaluate(&json!({ "process_name": "bash_shell" }), &ctx));
    assert!(condition.evaluate(&json!({ "username": "admin_user" }), &ctx));
    assert!(!condition.evaluate(&json!({ "username": "bob" }), &ctx));
}

#[test]
fn or_with_no_match_returns_false() {
    let ctx = make_ctx(HashMap::new());
    let condition = Condition::Or(
        Box::new(Condition::Filter("missing".into())),
        Box::new(Condition::Filter("Missing".into())),
    );
    assert!(!condition.evaluate(&json!({}), &ctx));
}
