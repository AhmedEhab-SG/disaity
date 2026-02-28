use serenity::all::{ActivityData, Context};
use std::time::Duration;
use tokio::time::sleep;

pub fn start_status_loop(ctx: Context) {
    tokio::spawn(async move {
        // Map your JSON array directly into Serenity ActivityData
        let statuses = vec![
            ActivityData::playing("hide & seek with Puck"), // Type 0
            ActivityData::watching("a Man becomes a Hero"), // Type 3
            ActivityData::listening("stories with Rem & Ram"), // Type 2
        ];

        let mut index = 0;

        loop {
            // Set the bot's current activity
            ctx.set_activity(Some(statuses[index].clone()));

            // Move to the next status, wrapping back to 0 at the end of the list
            index = (index + 1) % statuses.len();

            // Wait 15 minutes (15 * 60 seconds) before changing again
            sleep(Duration::from_secs(15 * 60)).await;
        }
    });
}
