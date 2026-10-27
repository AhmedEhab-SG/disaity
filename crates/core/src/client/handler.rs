use std::sync::Arc;

use serenity::{
    all::{Context as SerenityContext, FullEvent},
    async_trait,
    gateway::ShardManager,
};

use super::{Data, Error};

pub struct HandlerCx<'a> {
    pub serenity: &'a SerenityContext,
    pub shard_manager: Arc<ShardManager>,
    pub data: &'a Data,
}

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    /// runs once, after the bot is ready and the commands are registered
    async fn setup(&self, _cx: &HandlerCx<'_>) -> Result<(), Error> {
        Ok(())
    }

    /// runs for every discord event after setup, match the ones you want
    async fn event(&self, _cx: &HandlerCx<'_>, _event: &FullEvent) -> Result<(), Error> {
        Ok(())
    }
}
