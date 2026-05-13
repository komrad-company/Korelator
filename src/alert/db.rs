use serde_json::Value;
use sqlx::PgPool;
use tokio::sync::mpsc;

use khronika::error;
use konnect::{
    Uuid,
    chrono::{DateTime, Utc},
};

use crate::alert::Alert;

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

impl AlertRow {
    pub fn spawn_persist_task(pool: PgPool) -> mpsc::UnboundedSender<Alert> {
        let (tx, mut rx) = mpsc::unbounded_channel::<Alert>();

        tokio::spawn(async move {
            while let Some(alert) = rx.recv().await {
                if let Err(e) = AlertRow::insert(&pool, &alert).await {
                    error!("Failed to persist alert: {e}");
                }
            }
        });

        tx
    }
}
