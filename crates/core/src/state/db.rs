use std::fs::create_dir_all;

use sqlx::{Error, SqlitePool, sqlite::SqliteConnectOptions};

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn connect(db_path: &str) -> Result<Self, Error> {
        create_dir_all(db_path).ok();
        let file = format!("{db_path}/disaity.db");

        let opts = SqliteConnectOptions::new()
            .filename(&file)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(opts).await?;

        Ok(Self { pool })
    }
}
