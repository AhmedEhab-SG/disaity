use std::sync::Arc;
use tokio::sync::RwLock;

pub mod prayer_manager;

#[derive(Clone, Debug, Default)]
pub struct Subscription {
    pub prayer_subscription: Arc<RwLock<prayer_manager::PrayerSubscription>>,
}

impl Subscription {
    pub fn new(db_path: &String) -> Self {
        Self {
            prayer_subscription: Arc::new(RwLock::new(prayer_manager::PrayerSubscription::new(
                &db_path,
            ))),
        }
    }
}
