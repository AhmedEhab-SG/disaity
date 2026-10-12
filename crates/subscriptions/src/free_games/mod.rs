mod add;
mod clear;
mod handler;
mod store;

use std::sync::Arc;

use poise::Command;

use disaity_core::{Data, Error, Handler, SubscriptionModule};

pub struct FreeGamesSubscription;

impl SubscriptionModule for FreeGamesSubscription {
    fn add(&self) -> Command<Data, Error> {
        add::free_games()
    }
    fn clear(&self) -> Command<Data, Error> {
        clear::clear_free_games()
    }
    fn handler(&self) -> Arc<dyn Handler> {
        Arc::new(handler::FreeGamesHandler)
    }
}
