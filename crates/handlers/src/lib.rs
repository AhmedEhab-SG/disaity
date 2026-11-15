mod status;
mod welcome;

pub use status::StatusHandler;
pub use welcome::WelcomeHandler;

use std::sync::Arc;

use disaity_core::{Client, Feature, Handler};

#[derive(Default)]
pub struct Handlers(Vec<Arc<dyn Handler>>);

impl Handlers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, handler: impl Handler) -> Self {
        self.0.push(Arc::new(handler));
        self
    }

    pub fn all() -> Self {
        Self::new().with(StatusHandler).with(WelcomeHandler)
    }
}

impl Feature for Handlers {
    fn handlers(&self) -> Vec<Arc<dyn Handler>> {
        self.0.clone()
    }
}

impl IntoIterator for Handlers {
    type Item = Arc<dyn Handler>;
    type IntoIter = std::vec::IntoIter<Arc<dyn Handler>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub trait HandlersExt {
    fn with_handler(self, handler: impl Handler) -> Self;

    fn with_handlers(self, handlers: impl IntoIterator<Item = Arc<dyn Handler>>) -> Self;
}

impl HandlersExt for Client {
    fn with_handler(mut self, handler: impl Handler) -> Self {
        self.handlers.push(Arc::new(handler));
        self
    }

    fn with_handlers(mut self, handlers: impl IntoIterator<Item = Arc<dyn Handler>>) -> Self {
        self.handlers.extend(handlers);
        self
    }
}
