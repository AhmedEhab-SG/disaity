mod add;
mod clear;
mod handler;
mod store;

use std::sync::Arc;

use poise::Command;

use disaity_core::{Data, Error, Handler, SubscriptionModule};

pub struct PrayerModule;

impl SubscriptionModule for PrayerModule {
    fn add(&self) -> Command<Data, Error> {
        add::prayer()
    }
    fn clear(&self) -> Command<Data, Error> {
        clear::clear_prayer()
    }
    fn handler(&self) -> Arc<dyn Handler> {
        Arc::new(handler::PrayerHandler)
    }
}
