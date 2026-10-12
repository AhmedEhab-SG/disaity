use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serenity::model::id::{ChannelId, GuildId, RoleId};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool, sqlite::SqliteRow};

use disaity_core::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FreeGamesStoreInfo {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub role_id: Option<RoleId>,
}

#[derive(Clone, Debug)]
pub struct FreeGamesStore {
    pool: SqlitePool,
}

impl FreeGamesStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn init(&self) -> Result<(), Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS free_games_subscriptions (
                guild_id    TEXT PRIMARY KEY NOT NULL,
                channel_id  TEXT NOT NULL,
                role_id     TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS free_games_sent (
                guild_id     TEXT NOT NULL,
                giveaway_id  INTEGER NOT NULL,
                PRIMARY KEY (guild_id, giveaway_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    fn from_row(row: &SqliteRow) -> FreeGamesStoreInfo {
        FreeGamesStoreInfo {
            guild_id: GuildId::new(row.get::<String, _>("guild_id").parse::<u64>().unwrap_or(0)),
            channel_id: ChannelId::new(
                row.get::<String, _>("channel_id")
                    .parse::<u64>()
                    .unwrap_or(0),
            ),
            role_id: row
                .get::<Option<String>, _>("role_id")
                .and_then(|r| r.parse::<u64>().ok().map(RoleId::new)),
        }
    }

    pub async fn get_all(&self) -> Result<Vec<FreeGamesStoreInfo>, Error> {
        let rows =
            sqlx::query("SELECT guild_id, channel_id, role_id FROM free_games_subscriptions")
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.iter().map(Self::from_row).collect())
    }

    // pub async fn get(&self, guild_id: GuildId) -> Result<Option<FreeGamesStoreInfo>, Error> {
    //     let row = sqlx::query(
    //         "SELECT channel_id, role_id, country, city FROM free_games_subscriptions WHERE guild_id = ?",
    //     )
    //     .bind(guild_id.to_string())
    //     .fetch_optional(&self.pool)
    //     .await?;
    //
    //     Ok(row.as_ref().map(Self::from_row))
    // }

    pub async fn create(&self, info: FreeGamesStoreInfo) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO free_games_subscriptions (guild_id, channel_id, role_id)
            VALUES (?, ?, ?)
            ON CONFLICT(guild_id) DO UPDATE SET
                channel_id = excluded.channel_id,
                role_id = excluded.role_id
            "#,
        )
        .bind(info.guild_id.get().to_string())
        .bind(info.channel_id.get().to_string())
        .bind(info.role_id.map(|r| r.get().to_string()))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, guild_id: GuildId) -> Result<bool, Error> {
        let res = sqlx::query("DELETE FROM free_games_subscriptions WHERE guild_id = ?")
            .bind(guild_id.get().to_string())
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM free_games_sent WHERE guild_id = ?")
            .bind(guild_id.get().to_string())
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected() > 0)
    }

    pub async fn get_sent(&self, guild_id: GuildId) -> Result<HashSet<i64>, Error> {
        let rows = sqlx::query("SELECT giveaway_id FROM free_games_sent WHERE guild_id = ?")
            .bind(guild_id.get().to_string())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.iter().map(|row| row.get("giveaway_id")).collect())
    }

    pub async fn mark_sent(&self, guild_id: GuildId, giveaway_id: i64) -> Result<(), Error> {
        sqlx::query("INSERT OR IGNORE INTO free_games_sent (guild_id, giveaway_id) VALUES (?, ?)")
            .bind(guild_id.get().to_string())
            .bind(giveaway_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn prune_sent(&self, active: &[i64]) -> Result<(), Error> {
        if active.is_empty() {
            return Ok(());
        }

        let mut query =
            QueryBuilder::<Sqlite>::new("DELETE FROM free_games_sent WHERE giveaway_id NOT IN (");

        let mut ids = query.separated(", ");

        for id in active {
            ids.push(*id);
        }

        ids.push_unseparated(")");

        query.build().execute(&self.pool).await?;

        Ok(())
    }
}
