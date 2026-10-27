use std::{sync::Arc, sync::Mutex, time::Duration};

use serenity::{
    all::{Cache, ChannelId, Guild, GuildId, UserId},
    async_trait,
};
use songbird::{
    Call, CoreEvent, Event, EventContext, EventHandler, Songbird, TrackEvent, tracks::TrackQueue,
};
use tokio::{task::JoinHandle, time::sleep};

use super::{RegisterVoiceEvent, VoiceEventCtx};

/// create an idle event to disconnect the bot automatic when not needed
#[derive(Clone)]
struct IdleCheck {
    timer: IdleTimer,
    manager: Arc<Songbird>,
    guild_id: GuildId,
    cache: Arc<Cache>,
    queue: TrackQueue,
}

#[async_trait]
impl EventHandler for IdleCheck {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        self.run();
        None
    }
}

impl IdleCheck {
    fn run(&self) {
        if !self.is_idle() {
            self.timer.disarm();
            return;
        }

        self.timer.arm(self.manager.clone(), self.guild_id);
    }

    /// Idle = nothing queued, or no user left in the channel.
    fn is_idle(&self) -> bool {
        let bot_id = self.cache.current_user().id;

        let Some(guild) = self.cache.guild(self.guild_id) else {
            return false;
        };

        let Some(channel_id) = guild.voice_states.get(&bot_id).and_then(|vs| vs.channel_id) else {
            return false;
        };

        // using the same guild to
        // avoid looking up for the guild again from CacheRef
        self.queue.is_empty() || !Self::is_user_in(&guild, channel_id, bot_id)
    }

    fn is_user_in(guild: &Guild, channel_id: ChannelId, bot_id: UserId) -> bool {
        guild.voice_states.values().any(|vs| {
            let is_bot = vs.user_id == bot_id || matches!(&vs.member, Some(m) if m.user.bot);

            vs.channel_id == Some(channel_id) && !is_bot
        })
    }
}

pub(super) struct IdleEvent;

impl IdleEvent {
    const CHECK_EVERY: Duration = Duration::from_secs(IdleTimer::IDLE_TIMEOUT.as_secs() / 10);
}

impl RegisterVoiceEvent for IdleEvent {
    async fn register(call: &mut Call, cx: &VoiceEventCtx) {
        let check = IdleCheck {
            timer: IdleTimer::default(),
            manager: cx.manager.clone(),
            guild_id: cx.guild_id,
            cache: cx.cache.clone(),
            queue: call.queue().clone(),
        };

        // init counting
        check.timer.arm(check.manager.clone(), check.guild_id);

        call.add_global_event(Event::Track(TrackEvent::Play), check.clone());
        call.add_global_event(Event::Track(TrackEvent::End), check.clone());
        call.add_global_event(Event::Core(CoreEvent::ClientDisconnect), check.clone());
        call.add_global_event(Event::Periodic(Self::CHECK_EVERY, None), check);
    }
}

#[derive(Clone, Default)]
struct IdleTimer(Arc<Mutex<Option<JoinHandle<()>>>>);

impl IdleTimer {
    const IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

    /// Start the countdown, unless one is already running.
    fn arm(&self, manager: Arc<Songbird>, guild_id: GuildId) {
        let mut slot = self.0.lock().unwrap();

        if slot.as_ref().is_some_and(|task| !task.is_finished()) {
            return;
        }

        *slot = Some(tokio::spawn(async move {
            sleep(Self::IDLE_TIMEOUT).await;

            manager.remove(guild_id).await.ok();
        }));
    }

    fn disarm(&self) {
        if let Some(task) = self.0.lock().unwrap().take() {
            task.abort();
        }
    }
}
