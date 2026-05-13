use konnect::{PgPool, Uuid};

use super::AlertRow;

use crate::Error;

impl AlertRow {
    pub async fn find_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Option<AlertRow>, Error> {
        sqlx::query_as::<_, AlertRow>(
            "select id, rule_id, title, level, event, triggered_at
             from alerts where id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| Error::DatabaseError(konnect::Error::ConnectionError(e)))
    }
}
