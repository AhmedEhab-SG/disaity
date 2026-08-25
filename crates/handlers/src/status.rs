use serenity::{all::ActivityData, async_trait, gateway::ShardManager};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

use disaity_core::{Error, Feature, Handler, HandlerCx};

pub struct StatusFeature;

impl Feature for StatusFeature {
    fn handler(&self) -> Option<Arc<dyn Handler>> {
        Some(Arc::new(StatusHandler))
    }
}

pub struct StatusHandler;

#[async_trait]
impl Handler for StatusHandler {
    async fn setup(&self, cx: &HandlerCx<'_>) -> Result<(), Error> {
        start_status_loop(cx.shared_manager.clone());
        Ok(())
    }
}

fn start_status_loop(shard_manager: Arc<ShardManager>) {
    tokio::spawn(async move {
        // Map your JSON array directly into Serenity ActivityData
        let statuses = vec![
            ActivityData::playing("hide & seek with Puck"), // Type 0
            ActivityData::watching("a Man becomes a Hero"), // Type 3
            ActivityData::listening("stories with Rem & Ram"), // Type 2
        ];

        let mut index = 0;
        let mut minutes_passed = 0;

        loop {
            // Set the bot's current activity
            let runners = shard_manager.runners.lock().await;

            // Loop through every active shard runner and update its activity
            for runner in runners.values() {
                runner.runner_tx.set_activity(Some(statuses[index].clone()));
            }

            // Explicitly drop the lock so we aren't holding it while sleeping
            drop(runners);

            sleep(Duration::from_secs(60)).await;
            minutes_passed += 1;

            // Only switch status text when 15 minutes are up
            if minutes_passed >= 15 {
                index = (index + 1) % statuses.len();
                minutes_passed = 0;
            }
        }
    });
}
