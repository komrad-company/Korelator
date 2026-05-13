use konnect::{PgPool, Store};

use crate::Error;

pub struct AlertStore {
    pool: PgPool,
}

impl Store for AlertStore {
    fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl AlertStore {
    pub async fn migrate(&self) -> Result<(), Error> {
        sqlx::migrate!("./migrations")
            .run(self.pool())
            .await
            .map_err(Error::MigrationError)
    }
}
