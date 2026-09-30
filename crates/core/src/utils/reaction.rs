use serenity::{all::ReactionType, async_trait};

use crate::{context::Context, errors::Error};

use super::Utils;

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

    async fn on_error_react(
        &self,
        emojis: Option<&[impl Into<ReactionType> + Clone + Send + Sync]>,
    ) -> Result<(), Error>;
}

#[async_trait]
impl ReactionUtils for Utils<'_> {
    async fn add_reactions(
        &self,
        emojis: &[impl Into<ReactionType> + Clone + Send + Sync],
    ) -> Result<(), Error> {
        if let Context::Prefix(p_ctx) = self.ctx {
            for emoji in emojis {
                p_ctx.msg.react(self.ctx, emoji.clone()).await?;
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

    async fn on_error_react(
        &self,
        emojis: Option<&[impl Into<ReactionType> + Clone + Send + Sync]>,
    ) -> Result<(), Error> {
        self.delete_all_self_reactions().await?;
        match emojis {
            Some(emojis) => self.add_reactions(emojis).await?,
            None => self.add_reactions(&['❌']).await?,
        }
        Ok(())
    }
}

impl Utils<'_> {
    pub async fn start_loading_react(&self) -> Result<(), Error> {
        self.add_reactions(&['🔃']).await?;
        Ok(())
    }

    pub async fn end_loading_react(&self) -> Result<(), Error> {
        self.delete_self_reactions(&['🔃']).await?;
        self.add_reactions(&['✅']).await?;
        Ok(())
    }
}
