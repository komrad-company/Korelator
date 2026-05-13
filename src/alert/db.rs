use serde_json::Value;

use konnect::{
    Uuid,
    chrono::{DateTime, Utc},
};

mod reader;
mod writer;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AlertRow {
    pub id: Uuid,
    pub rule_id: String,
    pub title: String,
    pub level: String,
    pub event: Value,
    pub triggered_at: DateTime<Utc>,
}
