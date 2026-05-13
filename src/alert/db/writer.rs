use konnect::PgPool;

use super::AlertRow;

use crate::alert::Alert;
use crate::errors::Error;

impl AlertRow {
    pub async fn insert(pool: &PgPool, alert: &Alert) -> Result<AlertRow, Error> {
        sqlx::query_as::<_, AlertRow>(
            "INSERT INTO alerts (rule_id, title, level, event, triggered_at)
             VALUES ($1, $2, $3, $4, NOW())
             RETURNING id, rule_id, title, level, event, triggered_at",
        )
        .bind(&alert.rule_id)
        .bind(&alert.title)
        .bind(&alert.level)
        .bind(&alert.event)
        .fetch_one(pool)
        .await
        .map_err(|e| Error::DatabaseError(konnect::Error::ConnectionError(e)))
    }
}
