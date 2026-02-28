use serenity::{all::GuildId, async_trait};
use songbird::{Call, Event, EventContext, EventHandler, Songbird, TrackEvent};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle, time::sleep};

type TimeoutHandle = Arc<Mutex<Option<JoinHandle<()>>>>;

async fn cancel_timer(timeout: &TimeoutHandle) {
    let mut lock = timeout.lock().await;
    if let Some(task) = lock.take() {
        task.abort();
    };
}

async fn reset_timer(timeout: &TimeoutHandle, manager: Arc<Songbird>, guild_id: GuildId) {
    cancel_timer(timeout).await;
    let handle_clone = timeout.clone();

    let task = tokio::spawn(async move {
        sleep(Duration::from_secs(60)).await;

        manager.remove(guild_id).await.ok();

        let mut lock = handle_clone.lock().await;

        *lock = None;
    });

    let mut lock = timeout.lock().await;
    *lock = Some(task);
}

struct PlayHandler {
    pub timeout: TimeoutHandle,
}

#[async_trait]
impl EventHandler for PlayHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        cancel_timer(&self.timeout).await;

        dbg!("play event fired");

        None
    }
}

struct EndHandler {
    pub timeout: TimeoutHandle,
    pub manager: Arc<Songbird>,
    pub guild_id: GuildId,
}

#[async_trait]
impl EventHandler for EndHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let is_empty = if let Some(call) = self.manager.get(self.guild_id) {
            let call = call.lock().await;
            call.queue().is_empty()
        } else {
            false
        };

        dbg!("endhander", is_empty);

        if is_empty {
            reset_timer(&self.timeout, self.manager.clone(), self.guild_id).await;
        }

        None
    }
}

pub async fn register_idle_timeout(call: &mut Call, guild_id: GuildId, manager: Arc<Songbird>) {
    let timeout: TimeoutHandle = Arc::new(Mutex::new(None));

    // Start a timer immediately in case the bot joins but nothing is ever queued
    reset_timer(&timeout, manager.clone(), guild_id).await;

    // Hook up the cancellation on play
    call.add_global_event(
        Event::Track(TrackEvent::Play),
        PlayHandler {
            timeout: timeout.clone(),
        },
    );

    // Hook up the restart on end
    call.add_global_event(
        Event::Track(TrackEvent::End),
        EndHandler {
            timeout,
            manager,
            guild_id,
        },
    );
}
