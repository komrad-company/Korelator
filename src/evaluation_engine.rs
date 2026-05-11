use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

use kompiler::rules::filter::Filters;

mod condition;
mod filter;
mod rule;

pub use rule::PreparedRule;

pub struct EvaluationContext {
    pub filters: Arc<HashMap<String, Filters>>,
}

impl EvaluationContext {
    pub fn new(filters: HashMap<String, Filters>) -> Self {
        Self {
            filters: Arc::new(filters),
        }
    }
}
pub trait Evaluate {
    fn evaluate(&self, event: &Value, ctx: &EvaluationContext) -> bool;
}

#[cfg(test)]
mod tests;
