use serenity::{all::ReactionType, async_trait};
use songbird::{
    Call,
    input::{Input, YoutubeDl},
    tracks::Track,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    config::interactions::Provider,
    core::{context::Context, error::Error},
    handlers::{SongMetadata, register_all},
};

pub struct Utils<'a> {
    pub ctx: Context<'a>,
}

#[async_trait]
pub trait ReactionUtils {
    async fn add_reactions(
        &self,
        emojis: &[impl Into<ReactionType> + Clone + Send + Sync],
    ) -> Result<(), Error>;

    async fn delete_self_reactions(
        &self,
        emojis: &[impl Into<ReactionType> + Clone + Send + Sync],
    ) -> Result<(), Error>;

    async fn delete_all_self_reactions(&self) -> Result<(), Error>;

    async fn start_loading_react(&self) -> Result<(), Error>;
    async fn end_loading_react(&self) -> Result<(), Error>;
}

#[async_trait]
pub trait VoiceUtils {
    async fn get_or_join_voice(&self) -> Result<Arc<Mutex<Call>>, Error>;
}

#[async_trait]
pub trait ProviderUtils {
    async fn spotify_extractor(&self, url: &str) -> Result<String, Error>;
    async fn play(&self, song: String) -> Result<(Track, SongMetadata), Error>;
}

#[async_trait]
impl ReactionUtils for Utils<'_> {
    async fn add_reactions(
        &self,
        emojis: &[impl Into<ReactionType> + Clone + Send + Sync],
    ) -> Result<(), Error> {
        if let Context::Prefix(p_ctx) = self.ctx {
            for emoji in emojis {
                p_ctx.msg.react(self.ctx, emoji.clone().into()).await?;
            }
        }
        Ok(())
    }
    async fn delete_self_reactions(
        &self,
        emojis: &[impl Into<ReactionType> + Clone + Send + Sync],
    ) -> Result<(), Error> {
        if let Context::Prefix(p_ctx) = self.ctx {
            let target_emojis: Vec<ReactionType> =
                emojis.iter().map(|e| e.clone().into()).collect();

            let updated_msg = self
                .ctx
                .channel_id()
                .message(self.ctx, p_ctx.msg.id)
                .await?;

            for reaction in &updated_msg.reactions {
                if !reaction.me {
                    continue;
                }

                if target_emojis.contains(&reaction.reaction_type) {
                    updated_msg
                        .delete_reaction(self.ctx, None, reaction.reaction_type.clone())
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn start_loading_react(&self) -> Result<(), Error> {
        self.add_reactions(&['🔃']).await?;
        Ok(())
    }

    async fn end_loading_react(&self) -> Result<(), Error> {
        self.delete_self_reactions(&['🔃']).await?;
        self.add_reactions(&['✅']).await?;
        Ok(())
    }

    async fn delete_all_self_reactions(&self) -> Result<(), Error> {
        if let Context::Prefix(p_ctx) = self.ctx {
            let updated_msg = self
                .ctx
                .channel_id()
                .message(self.ctx, p_ctx.msg.id)
                .await?;

            for reaction in &updated_msg.reactions {
                if !reaction.me {
                    continue;
                }

                updated_msg
                    .delete_reaction(self.ctx, None, reaction.reaction_type.clone())
                    .await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl VoiceUtils for Utils<'_> {
    async fn get_or_join_voice(&self) -> Result<Arc<Mutex<Call>>, Error> {
        let serenity_context = self.ctx.serenity_context();
        let guild_id = self
            .ctx
            .guild_id()
            .ok_or("This command only works in servers.")?;
        let voice_channel_id = self
            .ctx
            .guild()
            .and_then(|g| {
                g.voice_states
                    .get(&self.ctx.author().id)
                    .and_then(|vs| vs.channel_id)
            })
            .ok_or("You must be in a voice channel!")?;
        let manager = songbird::get(serenity_context)
            .await
            .ok_or("Failed to mount songbird")?;

        let (call, is_new_call) = if let Some(exisiting_call) = manager.get(guild_id) {
            let mut call_lock = exisiting_call.lock().await;
            call_lock.join(voice_channel_id).await.ok();
            drop(call_lock);

            (exisiting_call, false)
        } else {
            let new_call = manager.join(guild_id, voice_channel_id).await?;
            (new_call, true)
        };

        if is_new_call {
            let mut call_lock = call.lock().await;

            register_all(
                &mut call_lock,
                call.clone(),
                guild_id,
                self.ctx.channel_id(),
                serenity_context.http.clone(),
                manager,
                serenity_context.cache.clone(),
            )
            .await;
        }

        Ok(call)
    }
}

#[async_trait]
impl ProviderUtils for Utils<'_> {
    async fn spotify_extractor(&self, url: &str) -> Result<String, Error> {
        let res = self.ctx.data().http.get(url).send().await?.text().await?;

        let title = res
            .find("<title>")
            .map(|start| start + 7)
            .and_then(|start_idx| {
                res[start_idx..]
                    .find("</title>")
                    .map(|end_idx| &res[start_idx..start_idx + end_idx])
            })
            .ok_or("failed to get extract the song title")?
            .replace(" - song and lyrics by", "")
            .replace(" | Spotify", "");

        Ok(title)
    }

    async fn play(&self, song: String) -> Result<(Track, SongMetadata), Error> {
        let original_url = song.clone();

        let (query, do_search, provider) = if song.contains("spotify.com") {
            (
                self.spotify_extractor(&song).await?,
                true,
                Provider::Spotify,
            )
        } else {
            let needs_search = !song.starts_with("http");
            let prov = if needs_search {
                Provider::Unknown
            } else {
                Provider::from_url(&song)
            };

            (song, needs_search, prov)
        };

        let mut src: Input = if do_search {
            YoutubeDl::new_search(self.ctx.data().http.clone(), query).into()
        } else {
            YoutubeDl::new(self.ctx.data().http.clone(), query).into()
        };

        let mut metadata = src.aux_metadata().await?;
        let author = self.ctx.author();

        let url = if provider == Provider::Spotify {
            original_url
        } else {
            metadata
                .source_url
                .take()
                .unwrap_or_else(|| "https://youtube.com".to_string())
        };

        let provider_logo = if provider == Provider::Unknown {
            Provider::from_url(&url)
        } else {
            provider
        };

        let info = SongMetadata {
            title: metadata
                .title
                .take()
                .unwrap_or_else(|| "Unknown".to_string()),
            url,
            thumbnail: metadata.thumbnail.take().unwrap_or_default(),
            duration: metadata.duration,

            request_by: author.name.clone(),
            request_by_avatar: author
                .avatar_url()
                .unwrap_or_else(|| author.default_avatar_url()),

            author: metadata
                .channel
                .take()
                .unwrap_or_else(|| "Unknown Author".to_string()),

            provider_logo_url: self
                .ctx
                .data()
                .config
                .interactions_registry
                .get_logo(provider_logo),
        };

        let track = Track::new_with_data(src.into(), Arc::new(info.clone()));

        Ok((track, info))
    }
}
