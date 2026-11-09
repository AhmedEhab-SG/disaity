mod add;
mod remove;

use poise::command;

use disaity_core::{Context, Error};

use add::add;
use remove::remove;

// doesnt run on its own
/// gives and takes roles
#[command(
    slash_command,
    prefix_command,
    guild_only,
    subcommand_required,
    subcommands("add", "remove")
)]
pub async fn role(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
