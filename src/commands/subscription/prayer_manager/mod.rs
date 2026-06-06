use std::collections::HashMap;

use serenity::model::id::{ChannelId, GuildId, RoleId};

pub mod clear_prayer;
pub mod prayer;

#[derive(Clone, Debug)]
pub struct PrayerSubscriptionInfo {
    pub channel_id: ChannelId,
    pub role_id: Option<RoleId>,
    pub city: String,
    pub country: String,
}

#[derive(Clone, Debug, Default)]
pub struct PrayerSubscription {
    pub subscription: HashMap<GuildId, PrayerSubscriptionInfo>,
}

impl PrayerSubscription {
    pub fn new() -> Self {
        Self {
            subscription: HashMap::new(),
        }
    }

    pub fn add_subscription(&mut self, guild_id: GuildId, info: PrayerSubscriptionInfo) {
        self.subscription.insert(guild_id, info);
    }

    pub fn remove_subscription(&mut self, guild_id: GuildId) -> Option<PrayerSubscriptionInfo> {
        self.subscription.remove(&guild_id)
    }

    pub fn get_subscription(&self, guild_id: GuildId) -> Option<&PrayerSubscriptionInfo> {
        self.subscription.get(&guild_id)
    }
}
