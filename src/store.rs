use tokio::sync::mpsc;

use khronika::error;
use konnect::{PgPool, Store};

use crate::{Alert, AlertRow, Error};

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
    pub async fn setup(config: &konnect::DatabaseConfig) -> Result<Self, Error> {
        let pool = konnect::init(config).await.map_err(Error::DatabaseError)?;
        let store = Self::new(pool);
        store.migrate().await?;
        Ok(store)
    }

    pub async fn migrate(&self) -> Result<(), Error> {
        sqlx::migrate!("./migrations")
            .run(self.pool())
            .await
            .map_err(Error::MigrationError)
    }

    pub fn spawn_persist_task(self) -> mpsc::UnboundedSender<Alert> {
        let (tx, mut rx) = mpsc::unbounded_channel::<Alert>();

        tokio::spawn(async move {
            while let Some(alert) = rx.recv().await {
                if let Err(e) = AlertRow::insert(&self.pool, &alert).await {
                    error!("Failed to persist alert: {e}");
                }
            }
        });

        tx
    }
}
