use std::{collections::HashMap, path::Path, sync::Arc};

use kompiler::Filters;
use serde_json::Value;

mod condition;
mod filter;
mod rule;

pub use rule::PreparedRule;

pub fn load_rules(path: &Path) -> Result<Vec<PreparedRule>, crate::Error> {
    Ok(kompiler::parse_rules(path)?
        .into_iter()
        .map(PreparedRule::from)
        .collect())
}

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
