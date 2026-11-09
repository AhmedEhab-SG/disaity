mod clear;
mod handler;
mod set;
mod status;
mod store;

use std::sync::Arc;

use disaity_config::CommandId;
use poise::Command;

use disaity_core::{Data, Error, Handler, SubscriptionModule};

pub struct FreeGamesSubscription;

impl SubscriptionModule for FreeGamesSubscription {
    fn name(&self) -> &'static str {
        CommandId::FreeGames.into()
    }
    fn set(&self) -> Command<Data, Error> {
        set::set()
    }
    fn clear(&self) -> Command<Data, Error> {
        clear::clear()
    }
    fn status(&self) -> Command<Data, Error> {
        status::status()
    }
    fn handler(&self) -> Arc<dyn Handler> {
        Arc::new(handler::FreeGamesHandler)
    }
}
