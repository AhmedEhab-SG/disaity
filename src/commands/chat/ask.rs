use poise::command;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command, rename = "ask", aliases("a"))]
pub async fn ask(ctx: Context<'_>, #[rest] msg: String) -> Result<(), Error> {
    dbg!(msg);

    Ok(())
}
