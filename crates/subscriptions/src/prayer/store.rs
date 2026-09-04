use serde::{Deserialize, Serialize};
use serenity::model::id::{ChannelId, GuildId, RoleId};
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};

use disaity_core::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrayerStoreInfo {
    pub channel_id: ChannelId,
    pub role_id: Option<RoleId>,
    pub city: String,
    pub country: String,
}

#[derive(Clone, Debug)]
pub struct PrayerStore {
    pool: SqlitePool,
}

impl PrayerStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn init(&self) -> Result<(), Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS prayer_subscriptions (
                guild_id    TEXT PRIMARY KEY NOT NULL,
                channel_id  TEXT NOT NULL,
                role_id     TEXT,
                city        TEXT NOT NULL,
                country     TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    fn from_row(row: &SqliteRow) -> PrayerStoreInfo {
        PrayerStoreInfo {
            channel_id: ChannelId::new(
                row.get::<String, _>("channel_id")
                    .parse::<u64>()
                    .unwrap_or(0),
            ),
            role_id: row
                .get::<Option<String>, _>("role_id")
                .and_then(|r| r.parse::<u64>().ok().map(RoleId::new)),
            city: row.get("city"),
            country: row.get("country"),
        }
    }

    pub async fn get_all(&self) -> Result<Vec<PrayerStoreInfo>, Error> {
        let rows =
            sqlx::query("SELECT channel_id, role_id, city, country FROM prayer_subscriptions")
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.iter().map(Self::from_row).collect())
    }

    // pub async fn get(&self, guild_id: GuildId) -> Result<Option<PrayerStoreInfo>, Error> {
    //     let row = sqlx::query(
    //         "SELECT channel_id, role_id, country, city FROM prayer_subscriptions WHERE guild_id = ?",
    //     )
    //     .bind(guild_id.to_string())
    //     .fetch_optional(&self.pool)
    //     .await?;
    //
    //     Ok(row.as_ref().map(Self::from_row))
    // }

    pub async fn create(&self, guild_id: GuildId, info: PrayerStoreInfo) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO prayer_subscriptions (guild_id, channel_id, role_id, city, country)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(guild_id) DO UPDATE SET
                channel_id = excluded.channel_id,
                role_id = excluded.role_id,
                city = excluded.city,
                country = excluded.country
            "#,
        )
        .bind(guild_id.get().to_string())
        .bind(info.channel_id.get().to_string())
        .bind(info.role_id.map(|r| r.get().to_string()))
        .bind(&info.city)
        .bind(&info.country)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, guild_id: GuildId) -> Result<bool, Error> {
        let res = sqlx::query("DELETE FROM prayer_subscriptions WHERE guild_id = ?")
            .bind(guild_id.get().to_string())
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected() > 0)
    }
}
