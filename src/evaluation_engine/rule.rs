use std::collections::HashMap;

use serde_json::Value;

use khronika::warn;
use kodeks::Alert;
use kompiler::RuleLevel;
use kompiler::{Condition, FieldFilter, Filters, Matcher, Rule};

use crate::evaluation_engine::{Evaluate, EvaluationContext};
/// A [`Rule`] with its filter context built once at load time.
///
/// Built from a parsed [`Rule`] via [`PreparedRule::from`]. The conversion flattens
/// `HashMap<String, Vec<Filters>>` (Kompiler's parsed shape) into the
/// `HashMap<String, Filters>` shape expected by [`EvaluationContext`].
pub struct PreparedRule {
    pub id: String,
    pub title: String,
    pub level: RuleLevel,
    matcher: Matcher,
    condition: Condition,
    context: EvaluationContext,
}

impl From<Rule> for PreparedRule {
    fn from(rule: Rule) -> Self {
        let filters: HashMap<String, Filters> = rule
            .filters
            .into_iter()
            .map(|(name, groups)| {
                let merged: Vec<FieldFilter> =
                    groups.into_iter().flat_map(|f| f.0.into_iter()).collect();
                (name, Filters(merged))
            })
            .collect();

        Self {
            id: rule.id,
            title: rule.title,
            level: rule.level,
            matcher: rule.matcher,
            condition: rule.condition,
            context: EvaluationContext::new(filters),
        }
    }
}

impl PreparedRule {
    /// Evaluates the rule against a single event.
    ///
    /// `Matcher::Single` returns the condition result directly.
    /// `Matcher::Threshold` is not yet implemented — it logs a warning and returns `false`.
    pub fn fires_on(&self, event: &Value) -> bool {
        match &self.matcher {
            Matcher::Single => self.condition.evaluate(event, &self.context),
            Matcher::Threshold { .. } => {
                warn!(
                    rule_id = self.id,
                    "Threshold matcher not yet implemented, event ignored"
                );
                false
            }
        }
    }

    pub fn to_alert(&self, event: Value) -> Alert {
        Alert::new(self.id.clone(), self.title.clone(), self.level.to_string(), event)
    }
}
