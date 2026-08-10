use serde::{Deserialize, Serialize};
use serenity::model::id::{ChannelId, GuildId, RoleId};
use sqlx::{Error, Row, SqlitePool, sqlite::SqliteRow};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrayerSubscriptionInfo {
    pub channel_id: ChannelId,
    pub role_id: Option<RoleId>,
    pub city: String,
    pub country: String,
}

#[derive(Clone, Debug)]
pub struct PrayerSubscription {
    pool: SqlitePool,
}

impl PrayerSubscription {
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

    fn into_info(&self, row: &SqliteRow) -> PrayerSubscriptionInfo {
        PrayerSubscriptionInfo {
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

    pub async fn get_all(&self) -> Result<Vec<PrayerSubscriptionInfo>, Error> {
        let rows =
            sqlx::query("SELECT channel_id, role_id, city, country FROM prayer_subscriptions")
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.into_iter().map(|row| self.into_info(&row)).collect())
    }

    pub async fn get(&self, guild_id: GuildId) -> Result<Option<PrayerSubscriptionInfo>, Error> {
        let row = sqlx::query(
            "SELECT channel_id, role_id, country, city FROM prayer_subscription WHERE guild_id = ?",
        )
        .bind(guild_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(|r| self.into_info(r)))
    }

    pub async fn create(
        &self,
        guild_id: GuildId,
        info: PrayerSubscriptionInfo,
    ) -> Result<(), Error> {
        sqlx::query(r#""#);
        todo!()
    }

    pub async fn delete() {
        todo!()
    }

    // fn save(&self) {
    //     let tmp = format!("{}.tmp", self.path);
    //     match serde_json::to_string_pretty(&self) {
    //         Ok(json) => {
    //             if let Err(e) = fs::write(&tmp, &json) {
    //                 tracing::error!("Failed to write tmp file: {e}");
    //                 return;
    //             }
    //
    //             if let Err(e) = fs::rename(&tmp, &self.path) {
    //                 tracing::error!("Failed to rename tmp file: {e}");
    //             }
    //         }
    //
    //         Err(e) => tracing::error!("Failed to serialize prayer subscriptions: {e}"),
    //     }
    // }
    //
    // pub fn get_subscription(&self, guild_id: GuildId) -> Option<&PrayerSubscriptionInfo> {
    //     self.subscription.get(&guild_id)
    // }
    //
    // pub fn add_subscription(&mut self, guild_id: GuildId, info: PrayerSubscriptionInfo) {
    //     self.subscription.insert(guild_id, info);
    //     self.save();
    // }
    //
    // pub fn remove_subscription(&mut self, guild_id: GuildId) -> Option<PrayerSubscriptionInfo> {
    //     let removed = self.subscription.remove(&guild_id);
    //     if removed.is_some() {
    //         self.save();
    //     }
    //     removed
    // }
}
