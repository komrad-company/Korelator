use serde_json::Value;

use kompiler::rules::filter::{FieldFilter, FilterTypes::*, Types};

use crate::evaluation_engine::{Evaluate, EvaluationContext};

impl Evaluate for FieldFilter {
    fn evaluate(&self, event: &Value, _ctx: &EvaluationContext) -> bool {
        let Some(event_val) = event.get(&self.field) else {
            return false;
        };
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
