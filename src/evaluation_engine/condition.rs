use kompiler::rules::condition::Condition;
use serde_json::Value;

use crate::evaluation_engine::{Evaluate, EvaluationContext};

impl Evaluate for Condition {
    fn evaluate(&self, event: &Value, ctx: &EvaluationContext) -> bool {
        match self {
            Condition::Or(left, rigth) => left.evaluate(event, ctx) || rigth.evaluate(event, ctx),
            Condition::And(left, rigth) => left.evaluate(event, ctx) && rigth.evaluate(event, ctx),
            Condition::Not(condition) => !condition.evaluate(event, ctx),
            Condition::Filter(name) => {
                let Some(filters) = ctx.filters.get(name) else {
                    return false;
                };
                // Filters est un Vec<FieldFilter>, OR implicite entre champs
                filters.0.iter().any(|ff| ff.evaluate(event, ctx))
            }
        }
    }
}
