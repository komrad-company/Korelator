use serde_json::Value;

use kompiler::rules::filter::{FieldFilter, FilterTypes::*, Types};

use crate::evaluation_engine::{Evaluate, EvaluationContext};

impl Evaluate for FieldFilter {
    fn evaluate(&self, event: &Value, _ctx: &EvaluationContext) -> bool {
        let Some(event_val) = event.get(&self.field) else {
            return false;
        };
        // Les values sont en OR implicite
        self.values
            .iter()
            .any(|expected| match (&self.condition, event_val, expected) {
                (Contains, Value::String(s), Types::String(v)) => s.contains(v.as_str()),
                (Startswith, Value::String(s), Types::String(v)) => s.starts_with(v.as_str()),
                (Endswith, Value::String(s), Types::String(v)) => s.ends_with(v.as_str()),
                (Exact, Value::String(s), Types::String(v)) => s == v,
                (Exact, Value::Number(n), Types::Integer(v)) => n.as_i64() == Some(*v),
                (Gt, Value::Number(n), Types::Integer(v)) => n.as_i64().is_some_and(|n| n > *v),
                (Gte, Value::Number(n), Types::Integer(v)) => n.as_i64().is_some_and(|n| n >= *v),
                (Lt, Value::Number(n), Types::Integer(v)) => n.as_i64().is_some_and(|n| n < *v),
                (Lte, Value::Number(n), Types::Integer(v)) => n.as_i64().is_some_and(|n| n <= *v),
                _ => false,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;
    use std::collections::HashMap;

    use kompiler::rules::filter::{FieldFilter, FilterTypes, Types};

    use crate::evaluation_engine::EvaluationContext;
    fn empty_ctx() -> EvaluationContext {
        EvaluationContext::new(HashMap::new())
    }

    fn make_filter(field: &str, condition: FilterTypes, values: Vec<Types>) -> FieldFilter {
        FieldFilter {
            field: field.into(),
            condition,
            values,
        }
    }

    #[test]
    fn contains_matches_substring() {
        let ff = make_filter(
            "process_name",
            FilterTypes::Contains,
            vec![Types::String("shell".into())],
        );
        assert!(ff.evaluate(&json!({ "process_name": "bash_shell" }), &empty_ctx()));
        assert!(!ff.evaluate(&json!({ "process_name": "nginx" }), &empty_ctx()));
    }

    #[test]
    fn contains_multiple_values_is_or() {
        let ff = make_filter(
            "process_name",
            FilterTypes::Contains,
            vec![Types::String("shell".into()), Types::String("sh".into())],
        );
        assert!(ff.evaluate(&json!({ "process_name": "bash_shell" }), &empty_ctx()));
        assert!(ff.evaluate(&json!({ "process_name": "zsh" }), &empty_ctx()));
        assert!(!ff.evaluate(&json!({ "process_name": "nginx" }), &empty_ctx()));
    }

    #[test]
    fn startswith_matches_prefix() {
        let ff = make_filter(
            "account",
            FilterTypes::Startswith,
            vec![Types::String("adm".into())],
        );
        assert!(ff.evaluate(&json!({ "account": "admin_svc" }), &empty_ctx()));
        assert!(!ff.evaluate(&json!({ "account": "user_adm" }), &empty_ctx()));
    }

    #[test]
    fn exact_integer_matches() {
        let ff = make_filter("id", FilterTypes::Exact, vec![Types::Integer(1)]);
        assert!(ff.evaluate(&json!({ "id": 1 }), &empty_ctx()));
        assert!(!ff.evaluate(&json!({ "id": 2 }), &empty_ctx()));
    }

    #[test]
    fn gt_integer_matches() {
        let ff = make_filter("id", FilterTypes::Gt, vec![Types::Integer(5)]);
        assert!(ff.evaluate(&json!({ "id": 6 }), &empty_ctx()));
        assert!(!ff.evaluate(&json!({ "id": 5 }), &empty_ctx()));
    }

    #[test]
    fn missing_field_returns_false() {
        let ff = make_filter(
            "username",
            FilterTypes::Contains,
            vec![Types::String("admin".into())],
        );
        assert!(!ff.evaluate(&json!({ "process_name": "shell" }), &empty_ctx()));
    }
}
