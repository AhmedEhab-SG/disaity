use serenity::{
    all::{Cache, ChannelId, GuildId},
    async_trait,
};
use songbird::{Call, CoreEvent, Event, EventContext, EventHandler, Songbird, TrackEvent};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle, time::sleep};

type TimeoutHandle = Arc<Mutex<Option<JoinHandle<()>>>>;

struct PlayHandler {
    pub timeout: TimeoutHandle,
}

struct EndHandler {
    pub timeout: TimeoutHandle,
    pub manager: Arc<Songbird>,
    pub guild_id: GuildId,
}

pub struct AloneHandler {
    pub timeout: TimeoutHandle,
    pub manager: Arc<Songbird>,
    pub guild_id: GuildId,
    pub cache: Arc<Cache>,
}

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
        sleep(Duration::from_secs(5 * 60)).await;

        manager.remove(guild_id).await.ok();

        let mut lock = handle_clone.lock().await;

        *lock = None;
    });

    let mut lock = timeout.lock().await;
    *lock = Some(task);
}

#[async_trait]
impl EventHandler for PlayHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        cancel_timer(&self.timeout).await;
        None
    }
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

        if is_empty {
            reset_timer(&self.timeout, self.manager.clone(), self.guild_id).await;
        }

        None
    }
}


#[async_trait]
impl EventHandler for AloneHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let is_alone = if let Some(call_lock) = self.manager.get(self.guild_id) {
            let call = call_lock.lock().await;
            
            if let Some(channel_id) = call.current_channel() {
                let channel_id = ChannelId::new(channel_id.0.get());
                
                // Count human users in the bot's current channel
                let count = self.cache.guild(self.guild_id).map(|g| {
                    g.voice_states
                        .values()
                        .filter(|vs| {
                            vs.channel_id == Some(channel_id) && 
                            // Make sure we aren't counting bots
                            !vs.member.as_ref().map(|m| m.user.bot).unwrap_or(false)
                        })
                        .count()
                }).unwrap_or(0);

                count == 0
            } else {
                false
            }
        } else {
            false
        };

        if is_alone {
            // No humans left! Start the 60s countdown.
            reset_timer(&self.timeout, self.manager.clone(), self.guild_id).await;
        } else {
            // Someone is in the channel (or joined). Cancel the disconnect.
            cancel_timer(&self.timeout).await;
        }

        None
    }
}

pub async fn register_idle_timeout(call: &mut Call, guild_id: GuildId, manager: Arc<Songbird>, cache: Arc<Cache>) {
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
            timeout: timeout.clone(),
            manager:manager.clone(),
            guild_id,
        },
    );
 
    call.add_global_event(
        Event::Core(CoreEvent::SpeakingStateUpdate),
        AloneHandler {
            timeout,
            manager,
            guild_id,
            cache
        },
    );
}
