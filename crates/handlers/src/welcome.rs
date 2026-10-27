use serenity::{
    all::{FullEvent, Mentionable},
    async_trait,
};

use disaity_core::{Error, Handler, HandlerCx};

/// greets new members in the server's system channel
pub struct WelcomeHandler;

#[async_trait]
impl Handler for WelcomeHandler {
    async fn event(&self, cx: &HandlerCx<'_>, event: &FullEvent) -> Result<(), Error> {
        let FullEvent::GuildMemberAddition { new_member } = event else {
            return Ok(());
        };

        let channel = new_member
            .guild_id
            .to_guild_cached(cx.serenity)
            .and_then(|guild| guild.system_channel_id);

        let line = cx
            .data
            .config
            .persona
            .interactions
            .events
            .guild
            .on_welcome
            .get_random_res();

        let (Some(channel), Some(line)) = (channel, line) else {
            return Ok(());
        };

        channel
            .say(cx.serenity, format!("{} {line}", new_member.mention()))
            .await?;

        Ok(())
    }
}
