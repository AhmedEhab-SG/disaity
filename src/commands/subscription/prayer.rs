use poise::command;
use serenity::model::id::{ChannelId, RoleId};

use crate::core::{context::Context, errors::Error};

#[command(
    slash_command,
    prefix_command,
    // broadcast_typing,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS | ADD_REACTIONS"
)]
pub async fn prayer(
    ctx: Context<'_>,

    #[description = "Set channel id wants to send the prayer time one"] channel_id: ChannelId,

    #[description = "Add role id that u want to ping! (Optional)"] role: Option<RoleId>,
) -> Result<(), Error> {
    Ok(())
}
