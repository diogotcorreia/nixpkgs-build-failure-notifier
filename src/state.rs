use anyhow::{Context, Result};
use keyv::{Keyv, adapter::postgres::PostgresStoreBuilder};
use serde::{Deserialize, Serialize};

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
        build_id: u64,
        build_status: u8,
    ) -> Result<Option<PreviousBuild>> {
        let old = self
            .conn
            .get(job_full_name)
            .await?
            .and_then(|value| serde_json::from_value(value).ok());

        self.conn
            .set(
                job_full_name,
                PreviousBuild {
                    id: build_id,
                    status: build_status,
                },
            )
            .await?;

        Ok(old)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PreviousBuild {
    pub id: u64,
    pub status: u8,
}

impl PreviousBuild {
    pub fn is_failing(&self) -> bool {
        self.status != 0
    }
}
