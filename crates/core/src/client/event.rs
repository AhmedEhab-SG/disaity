// TODO(serenity next): the type map, and poise's `event_handler` and `setup`, are all gone there.
// serenity's `EventHandler` gets a catch-all `dispatch(&self, ctx, &FullEvent)`, so one struct
// that holds the list replaces this file, registered with `ClientBuilder::event_handler`.
// It reads `Data` with `ctx.data::<Data>()` and runs every `setup` once, on the first `Ready`.

use std::sync::Arc;

use poise::FrameworkContext;
use serenity::{all::FullEvent, prelude::TypeMapKey};

use super::{Data, Error, Handler, HandlerCx};

/// the handler list, kept in serenity's per-bot type map where the event hook can reach it
pub(super) struct Handlers;

impl TypeMapKey for Handlers {
    type Value = Vec<Arc<dyn Handler>>;
}

/// poise's `event_handler`, every handler gets each event after poise is done with it
pub(super) async fn dispatch(
    framework: FrameworkContext<'_, Data, Error>,
    event: &FullEvent,
) -> Result<(), Error> {
    // copy the list out so the lock isn't held while the handlers run
    let handlers = framework
        .serenity_context
        .data
        .read()
        .await
        .get::<Handlers>()
        .cloned()
        .unwrap_or_default();

    let cx = HandlerCx {
        serenity: framework.serenity_context,
        shard_manager: framework.shard_manager(),
        data: framework.user_data,
    };

    for handler in &handlers {
        // one broken handler shouldn't stop the rest from seeing the event
        if let Err(error) = handler.event(&cx, event).await {
            tracing::warn!(event = event.snake_case_name(), "a handler failed: {error}");
        }
    }

    Ok(())
}

// // serenity calls this for every event, it holds the list itself
// pub(super) struct Events {
//     pub(super) handlers: Vec<Arc<dyn Handler>>,
//     ready: OnceLock<()>,
// }
//
// #[async_trait]
// impl EventHandler for Events {
//     async fn dispatch(&self, ctx: &SerenityContext, event: &FullEvent) {
//         let cx = HandlerCx {
//             serenity: ctx,
//             data: ctx.data_ref::<Data>(),
//         };
//
//         // Ready repeats after reconnects, setup only runs the first time
//         if matches!(event, FullEvent::Ready { .. }) && self.ready.set(()).is_ok() {
//             // today's setup closure body moves here:
//             // register the slash commands, then every handler's setup
//         }
//
//         for handler in &self.handlers {
//             if let Err(error) = handler.event(&cx, event).await {
//                 tracing::warn!(event = event.snake_case_name(), "a handler failed: {error}");
//             }
//         }
//     }
// }
//
// // in Client::run
// ClientBuilder::new(token, intents)
//     .framework(Box::new(Framework::new(options)))
//     .event_handler(Arc::new(Events { handlers, ready: OnceLock::new() }))
//     .data(Arc::new(data))
