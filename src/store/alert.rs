use konnect::{Error, PgPool, Store};

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

    async fn migrate(&self) -> Result<(), Error> {
        sqlx::migrate!("./migrations")
            .run(self.pool())
            .await
            .map_err(Error::MigrationError)
    }
}
