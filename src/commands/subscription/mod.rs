use std::sync::Arc;
use tokio::sync::RwLock;

pub mod prayer_manager;

#[derive(Clone, Debug, Default)]
pub struct Subscription {
    pub prayer_subscription: Arc<RwLock<prayer_manager::PrayerSubscription>>,
}

impl Subscription {
    pub fn new() -> Self {
        Self::default()
    }
}
