use konnect::PgPool;

use super::AlertRow;

use crate::alert::Alert;
use crate::errors::Error;

impl AlertRow {
    pub async fn insert(pool: &PgPool, alert: &Alert) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO alerts (rule_id, title, level, event, triggered_at)
             VALUES ($1, $2, $3, $4, NOW())",
        )
        .bind(&alert.rule_id)
        .bind(&alert.title)
        .bind(&alert.level)
        .bind(&alert.event)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| Error::DatabaseError(konnect::Error::ConnectionError(e)))
    }
}
