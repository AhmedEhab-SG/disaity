use std::sync::Arc;
use tokio::sync::RwLock;

mod prayer_manager;
pub use prayer_manager::{PrayerSubscription, clear_prayer, prayer};

#[derive(Clone, Debug, Default)]
pub struct Subscription {
    pub prayer_subscription: Arc<RwLock<PrayerSubscription>>,
}

impl Subscription {
    pub fn new(db_path: &String) -> Self {
        Self {
            prayer_subscription: Arc::new(RwLock::new(PrayerSubscription::new(&db_path))),
        }
    }
}
