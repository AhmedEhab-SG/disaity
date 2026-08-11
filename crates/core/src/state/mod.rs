mod db;
mod prayer;

pub use prayer::{PrayerSubscription, PrayerSubscriptionInfo};

use crate::{Error, state::db::Database};

#[derive(Clone, Debug)]
pub struct Subscription {
    pub prayer_subscription: PrayerSubscription,
}

impl Subscription {
    pub async fn connect(db_path: &str) -> Result<Self, Error> {
        let db = Database::connect(db_path).await?;

        let prayer_subscription = PrayerSubscription::new(db.pool.clone());

        prayer_subscription.init().await?;

        Ok(Self {
            prayer_subscription,
        })
    }
}
