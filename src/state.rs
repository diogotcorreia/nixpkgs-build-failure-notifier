use anyhow::{Context, Result};
use keyv::{Keyv, adapter::postgres::PostgresStoreBuilder};

pub struct BuildStore {
    conn: Keyv,
}

impl BuildStore {
    pub async fn new(db_url: &str) -> Result<Self> {
        let store = PostgresStoreBuilder::new()
            .uri(db_url)
            .table_name("last_build_status")
            .build()
            .await
            .context("failed to connect to database")?;
        let keyv = Keyv::try_new(store)
            .await
            .context("failed to initialize database")?;

        Ok(Self { conn: keyv })
    }

    /// Update the latest known build status in the database.
    /// This is used to know whether builds are newly failing.
    /// Returns the previously known build status.
    pub async fn update_build_status(
        &self,
        job_full_name: &str,
        build_status: u8,
    ) -> Result<Option<u8>> {
        let old = self
            .conn
            .get(job_full_name)
            .await?
            .and_then(|value| serde_json::from_value(value).ok());

        self.conn.set(job_full_name, build_status).await?;

        Ok(old)
    }
}
