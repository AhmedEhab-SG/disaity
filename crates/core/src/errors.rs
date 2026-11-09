use std::error::Error as ErrorStd;

use poise::FrameworkError;
use serenity::all::ReactionType;

use super::{
    context::{ContextExt, Data},
    say,
    utils::ReactionUtils,
};

pub type Error = Box<dyn ErrorStd + Send + Sync>;

pub struct ErrorExt;

// TODO add presona response reflects Log level
impl ErrorExt {
    fn tracing_error(error: &FrameworkError<'_, Data, Error>) {
        match &error {
            FrameworkError::UnknownCommand { .. }
            | FrameworkError::ArgumentParse { .. }
            | FrameworkError::CommandCheckFailed { .. }
            | FrameworkError::MissingUserPermissions { .. }
            | FrameworkError::CooldownHit { .. }
            | FrameworkError::SubcommandRequired { .. }
            | FrameworkError::NotAnOwner { .. }
            | FrameworkError::GuildOnly { .. }
            | FrameworkError::DmOnly { .. }
            | FrameworkError::NsfwOnly { .. } => tracing::debug!("{error}"),

            FrameworkError::Command { error: why, .. } => tracing::warn!("{error}: {why}"),

            FrameworkError::MissingBotPermissions { .. } => tracing::warn!("{error}"),

            FrameworkError::Setup { error: why, .. }
            | FrameworkError::EventHandler { error: why, .. } => tracing::error!("{error}: {why}"),

            _ => tracing::error!("{error}"),
        }
    }

    pub async fn on_error(error: FrameworkError<'_, Data, Error>) -> Result<(), Error> {
        Self::tracing_error(&error);

        match error {
            FrameworkError::CommandCheckFailed { ctx, error, .. } => {
                if let Some(err_msg) = error {
                    say!(ctx, err_msg.to_string());

                    ctx.utils().on_error_react(None::<&[char]>).await?;
                }
            }
            FrameworkError::Command { ctx, error, .. } => {
                say!(ctx, error.to_string());
            }

            FrameworkError::ArgumentParse {
                ctx, error, input, ..
            } => {
                say!(
                    ctx,
                    format!("Bad argument `{}`: {error}", input.unwrap_or_default())
                );

                ctx.utils()
                    .on_error_react(Some(&[ReactionType::Unicode("⁉️".into())]))
                    .await?;
            }

            FrameworkError::MissingBotPermissions {
                missing_permissions,
                ctx,
                ..
            } => {
                say!(
                    ctx,
                    format!("I'm missing permissions: {missing_permissions}")
                );

                ctx.utils().on_error_react(None::<&[char]>).await?;
            }

            FrameworkError::MissingUserPermissions {
                missing_permissions,
                ctx,
                ..
            } => {
                say!(
                    ctx,
                    format!(
                        "You're missing permissions: {}",
                        missing_permissions.map_or("unknown".into(), |p| p.to_string())
                    )
                );

                ctx.utils().on_error_react(None::<&[char]>).await?;
            }

            other => {
                let ctx = other.ctx().ok_or("Failed to fetch context")?;

                if let Err(e) = poise::builtins::on_error(other).await {
                    say!(ctx, format!("Unhandled framework error: {e}"));

                    ctx.utils().on_error_react(None::<&[char]>).await.ok();
                }
            }
        }

        Ok(())
    }

    /// the same failure a user would see, as plain text for the ai
    pub fn text(error: &FrameworkError<'_, Data, Error>) -> String {
        match error {
            FrameworkError::Command { error, .. } => error.to_string(),

            FrameworkError::CommandCheckFailed {
                error: Some(error), ..
            } => error.to_string(),

            FrameworkError::MissingUserPermissions {
                missing_permissions: Some(missing),
                ..
            } => format!("the requester is missing: {missing}"),

            FrameworkError::MissingBotPermissions {
                missing_permissions,
                ..
            } => format!("I'm missing: {missing_permissions}"),

            FrameworkError::ArgumentParse { error, .. } => format!("bad arguments: {error}"),
            other => other.to_string(),
        }
    }
}
