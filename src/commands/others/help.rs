use poise::{
    CreateReply, command,
    serenity_prelude::{CreateEmbed, CreateEmbedFooter},
};
use serenity::all::CreateEmbedAuthor;

use crate::core::{Context, Error};

#[command(slash_command, prefix_command, rename = "help", aliases("h"))]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let (avatar_url, author_name) = {
        let user = ctx.serenity_context().cache.current_user();

        (
            user.avatar_url()
                .unwrap_or_else(|| user.default_avatar_url()),
            format!("{} 💖", user.name),
        )
    };

    let embed = CreateEmbed::new()
        .title("Hello,")
        .description("\n I'm Emilia, you'll often find me indulging in my love for nature walks, admiring the beauty of flowers 🌸 I have a soft spot for sweet treats, especially anything with strawberries - they're just so delightful! 🍓✨.\nIf you're up for a chat feel free to ask me anything. 🗨\n\nAlso I could play and manage your favorite music. 🎵")
        .thumbnail(&avatar_url)
        .color(0x703be7)
        .author(
            CreateEmbedAuthor::new(author_name)
                .icon_url(avatar_url)
        )
        .footer(
            CreateEmbedFooter::new("build with love")
                .icon_url("https://i.ibb.co/hFNhYk2/AES-solid-colors-512.png")
        );

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
