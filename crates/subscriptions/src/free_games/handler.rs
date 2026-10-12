use std::time::Duration;

use chrono::NaiveDateTime;
use reqwest::Client;
use serde::Deserialize;
use serenity::async_trait;
use serenity::builder::{
    CreateActionRow, CreateAllowedMentions, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateMessage,
};
use serenity::{all::Context as SerenityContext, model::id::RoleId};

use disaity_config::{Assets, Platform};
use disaity_core::{Error, Handler, HandlerCx};

use super::store::{FreeGamesStore, FreeGamesStoreInfo};

pub struct FreeGamesHandler;

#[async_trait]
impl Handler for FreeGamesHandler {
    async fn setup(&self, cx: &HandlerCx<'_>) -> Result<(), Error> {
        let Some(db) = cx.data.db.as_ref() else {
            return Ok(());
        };

        let store = FreeGamesStore::new(db.pool.clone());
        store.init().await?;

        let color = cx.data.config.persona.interactions.colors.accent;
        let assets = cx.data.config.assets.clone();

        Self::start_free_games_loop(
            cx.serenity.clone(),
            store,
            cx.data.http.clone(),
            color,
            assets,
        );

        Ok(())
    }
}

#[derive(Deserialize)]
struct Giveaway {
    id: i64,
    title: String,
    worth: String,
    image: String,
    description: String,
    platforms: String,
    end_date: String,
    open_giveaway_url: String,
}

impl Giveaway {
    const GIVEAWAYS_URL: &str = "https://www.gamerpower.com/api/filter?platform=epic-games-store.steam.gog.ubisoft.origin.battlenet&type=game";

    pub async fn fetch(http: &Client) -> Option<Vec<Self>> {
        let res = http.get(Self::GIVEAWAYS_URL).send().await.ok()?;
        res.json::<Vec<Self>>().await.ok()
    }
}

impl FreeGamesHandler {
    const POLL_INTERVAL: Duration = Duration::from_secs(10 * 60);

    pub fn start_free_games_loop(
        ctx: SerenityContext,
        store: FreeGamesStore,
        http: Client,
        color: u32,
        assets: Assets,
    ) {
        tokio::spawn(async move {
            loop {
                if let Some(giveaways) = Giveaway::fetch(&http).await {
                    let active: Vec<i64> = giveaways.iter().map(|g| g.id).collect();

                    store.prune_sent(&active).await.ok();

                    let subscription = store.get_all().await.unwrap_or_default();

                    for sub in subscription {
                        Self::notify_guild(
                            &ctx,
                            &store,
                            &sub,
                            &giveaways,
                            color,
                            assets.to_owned(),
                        )
                        .await;
                    }
                }

                tokio::time::sleep(Self::POLL_INTERVAL).await;
            }
        });
    }

    async fn notify_guild(
        ctx: &SerenityContext,
        store: &FreeGamesStore,
        sub: &FreeGamesStoreInfo,
        giveaways: &[Giveaway],
        color: u32,
        assets: Assets,
    ) {
        // Skip on a DB error — an empty set here would re-post every game.
        let Ok(sent) = store.get_sent(sub.guild_id).await else {
            return;
        };

        let mut role_id = sub.role_id;

        // API lists newest first; reverse so older games post first.
        for giveaway in giveaways.iter().rev() {
            if sent.contains(&giveaway.id) {
                continue;
            }

            let message = Self::giveaway_message(giveaway, role_id, color, assets.to_owned());

            let result = sub.channel_id.send_message(&ctx.http, message).await;

            if let Ok(msg) = result {
                // Announcement channels: push the post to every server that follows it.
                msg.crosspost(&ctx.http).await.ok();

                // Ping the role once per batch
                role_id = None;
                store.mark_sent(sub.guild_id, giveaway.id).await.ok();
            }
        }
    }

    fn giveaway_message(
        giveaway: &Giveaway,
        role_id: Option<RoleId>,
        color: u32,
        assets: Assets,
    ) -> CreateMessage {
        let claim = CreateButton::new_link(&giveaway.open_giveaway_url).label("Claim");

        let mut message = CreateMessage::new()
            .embed(Self::giveaway_embed(giveaway, color, assets))
            .components(vec![CreateActionRow::Buttons(vec![claim])]);

        if let Some(role_id) = role_id {
            message = message
                .content(format!("<@&{role_id}>"))
                .allowed_mentions(CreateAllowedMentions::new().roles(vec![role_id]));
        }

        message
    }

    fn giveaway_embed(giveaway: &Giveaway, color: u32, assets: Assets) -> CreateEmbed {
        // "Fireside Feelings (Steam) Giveaway" → "Fireside Feelings"
        let title = giveaway
            .title
            .rsplit_once(" (")
            .map(|(name, _)| name)
            .or_else(|| giveaway.title.strip_suffix(" Giveaway"))
            .unwrap_or(&giveaway.title);

        let logo = assets.get_platform_logo(Platform::from_platforms(&giveaway.platforms));

        let price = match giveaway.worth.as_str() {
            "N/A" => "**Free**".to_string(),
            worth => format!("~~{worth}~~ **Free**"),
        };

        let until = NaiveDateTime::parse_from_str(&giveaway.end_date, "%Y-%m-%d %H:%M:%S")
            .map(|date| date.format("%b %-d, %Y").to_string())
            .unwrap_or_else(|_| "No end date".to_string());

        let description = giveaway
            .description
            .lines()
            .map(|line| format!("> {line}"))
            .collect::<Vec<_>>()
            .join("\n");

        CreateEmbed::new()
            .title(title)
            .url(&giveaway.open_giveaway_url)
            .description(description)
            .color(color)
            .field("Price", price, true)
            .field("Until", until, true)
            .field("Platforms", &giveaway.platforms, true)
            .thumbnail(logo)
            .image(&giveaway.image)
            .footer(CreateEmbedFooter::new("via GamerPower"))
    }
}
