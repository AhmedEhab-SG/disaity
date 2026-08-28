use serenity::{all::ActivityData, async_trait, gateway::ShardManager};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

use disaity_config::{ActivityType, Status};
use disaity_core::{Error, Handler, HandlerCx};

/// How long one status stays up before the next entry in the persona's list.
const ROTATE_EVERY: Duration = Duration::from_secs(15 * 60);

pub struct StatusHandler;

#[async_trait]
impl Handler for StatusHandler {
    async fn setup(&self, cx: &HandlerCx<'_>) -> Result<(), Error> {
        let statuses: Vec<ActivityData> = cx
            .data
            .config
            .persona
            .interactions
            .status
            .iter()
            .filter_map(Self::to_activity)
            .collect();

        if statuses.is_empty() {
            return Ok(());
        }

        Self::start_status_loop(cx.shared_manager.clone(), statuses);
        Ok(())
    }
}

impl StatusHandler {
    fn start_status_loop(shard_manager: Arc<ShardManager>, statuses: Vec<ActivityData>) {
        tokio::spawn(async move {
            for status in statuses.iter().cycle() {
                // Set the bot's current activity
                let runners = shard_manager.runners.lock().await;

                // Loop through every active shard runner and update its activity
                for runner in runners.values() {
                    runner.runner_tx.set_activity(Some(status.clone()));
                }

                // Explicitly drop the lock so we aren't holding it while sleeping
                drop(runners);

                sleep(ROTATE_EVERY).await;
            }
        });
    }

    fn to_activity(status: &Status) -> Option<ActivityData> {
        let name = &status.name;

        Some(match status.activity_type {
            ActivityType::Playing => ActivityData::playing(name),
            ActivityType::Listening => ActivityData::listening(name),
            ActivityType::Watching => ActivityData::watching(name),
            ActivityType::Competing => ActivityData::competing(name),
            ActivityType::Streaming => match status.url.as_deref() {
                Some(url) => ActivityData::streaming(name, url)
                    .inspect_err(|why| {
                        tracing::warn!("status {:?} has a bad url: {why}", status.name)
                    })
                    .ok()?,
                None => {
                    tracing::warn!("status {:?} is `streaming` but has no url", status.name);
                    return None;
                }
            },
        })
    }
}
