use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use serenity::model::id::{ChannelId, GuildId, RoleId};

pub mod clear_prayer;
pub mod prayer;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrayerSubscriptionInfo {
    pub channel_id: ChannelId,
    pub role_id: Option<RoleId>,
    pub city: String,
    pub country: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PrayerSubscription {
    #[serde(skip)]
    path: String,
    pub subscription: HashMap<GuildId, PrayerSubscriptionInfo>,
}

impl PrayerSubscription {
    pub fn new(db_path: &String) -> Self {
        let path = format!("{}/prayer_subscriptions.json", db_path);

        fs::create_dir_all(&db_path).unwrap_or_else(|e| {
            tracing::error!("failed to create prayer db: {e}");
        });

        if !Path::new(&path).exists() {
            return Self {
                path,
                ..Default::default()
            };
        }

        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str::<PrayerSubscription>(&contents)
                .map(|mut sub| {
                    sub.path = path.clone();
                    sub
                })
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to parse prayer subscriptions DB: {e}");
                    Self {
                        path,
                        ..Default::default()
                    }
                }),

            Err(e) => {
                tracing::error!("Failed to read prayer subscriptions DB: {e}");
                return Self {
                    path,
                    ..Default::default()
                };
            }
        }
    }

    fn save(&self) {
        let tmp = format!("{}.tmp", self.path);
        match serde_json::to_string_pretty(&self) {
            Ok(json) => {
                if let Err(e) = fs::write(&tmp, &json) {
                    tracing::error!("Failed to write tmp file: {e}");
                    return;
                }

                if let Err(e) = fs::rename(&tmp, &self.path) {
                    tracing::error!("Failed to rename tmp file: {e}");
                }
            }

            Err(e) => tracing::error!("Failed to serialize prayer subscriptions: {e}"),
        }
    }

    pub fn get_subscription(&self, guild_id: GuildId) -> Option<&PrayerSubscriptionInfo> {
        self.subscription.get(&guild_id)
    }

    pub fn add_subscription(&mut self, guild_id: GuildId, info: PrayerSubscriptionInfo) {
        self.subscription.insert(guild_id, info);
        self.save();
    }

    pub fn remove_subscription(&mut self, guild_id: GuildId) -> Option<PrayerSubscriptionInfo> {
        let removed = self.subscription.remove(&guild_id);
        if removed.is_some() {
            self.save();
        }
        removed
    }
}
