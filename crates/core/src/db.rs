use std::fs::create_dir_all;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use super::Error;

#[derive(Debug)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn connect(db_path: &str, persona_name: &str) -> Result<Self, Error> {
        create_dir_all(db_path)?;
        let file = format!("{db_path}/{persona_name}.db");

        let opts = SqliteConnectOptions::new()
            .filename(&file)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(opts).await?;

        Ok(Self { pool })
    }
}
