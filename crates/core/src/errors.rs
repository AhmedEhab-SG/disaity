use std::error::Error as ErrorStd;

use poise::FrameworkError;
use serenity::all::ReactionType;

use super::{
    context::{ContextExt, Data},
    say,
    utils::ReactionUtils,
};

pub type Error = Box<dyn ErrorStd + Send + Sync>;

pub struct ErrorHandler;

impl ErrorHandler {
    pub async fn on_error(error: FrameworkError<'_, Data, Error>) -> Result<(), Error> {
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
                let ctx = other.ctx();

                if let Err(e) = poise::builtins::on_error(other).await {
                    eprintln!("Unhandled framework error: {e}");
                    ctx.ok_or("failed to fetch context")?
                        .utils()
                        .on_error_react(None::<&[char]>)
                        .await
                        .ok();
                }
            }
        }

        Ok(())
    }
}
