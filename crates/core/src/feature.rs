use std::{str::FromStr, sync::Arc};

use disaity_config::{CommandId, CommandRegistry};
use poise::Command;

use crate::{Data, Error, Handler};

pub trait Feature: Send + Sync + 'static {
    fn enabled(&self, _data: &Data) -> bool {
        true
    }
    fn needs_db(&self) -> bool {
        false
    }
    fn needs_ai(&self) -> bool {
        false
    }
    fn commands(&self, _data: &Data) -> Vec<Command<Data, Error>> {
        Vec::new()
    }
    fn handler(&self) -> Option<Arc<dyn Handler>> {
        None
    }
}

pub trait Decorate {
    fn decorate(self, registry: &CommandRegistry) -> Self;
}

impl Decorate for Command<Data, Error> {
    fn decorate(mut self, registry: &CommandRegistry) -> Self {
        let Ok(id) = CommandId::from_str(self.name.as_str()) else {
            return self;
        };

        let entry = registry.get_command(&id);

        self.name = entry.name.clone();
        self.description = Some(entry.description.clone());
        self.aliases = entry.keys.clone();
        self.category = Some(entry.category.clone());
        self
    }
}

pub trait SubscriptionModule: Send + Sync + 'static {
    fn add(&self) -> Command<Data, Error>;
    fn clear(&self) -> Command<Data, Error>;
    fn handler(&self) -> Arc<dyn Handler>;
    fn enabled(&self, _data: &Data) -> bool {
        true
    }
}

pub struct AsSubscription<S: SubscriptionModule>(pub S);

impl<S: SubscriptionModule> Feature for AsSubscription<S> {
    fn needs_db(&self) -> bool {
        true
    }

    fn commands(&self, data: &Data) -> Vec<Command<Data, Error>> {
        let reg = &data.config.commands_registry;
        vec![self.0.add().decorate(reg), self.0.clear().decorate(reg)]
    }

    fn handler(&self) -> Option<Arc<dyn Handler>> {
        Some(self.0.handler())
    }
}

type Gate = Box<dyn Fn(&Data) -> bool + Send + Sync>;
type CommandFn = Box<dyn Fn() -> Command<Data, Error> + Send + Sync>;

#[derive(Default)]
pub struct FeatureBuilder {
    commands: Vec<CommandFn>,
    handler: Option<Arc<dyn Handler>>,
    gate: Option<Gate>,
    needs_ai: bool,
}

impl FeatureBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_command(
        mut self,
        ctor: impl Fn() -> Command<Data, Error> + Send + Sync + 'static,
    ) -> Self {
        self.commands.push(Box::new(ctor));
        self
    }

    pub fn with_commands<F>(mut self, ctors: impl IntoIterator<Item = F>) -> Self
    where
        F: Fn() -> Command<Data, Error> + Send + Sync + 'static,
    {
        for ctor in ctors {
            self.commands.push(Box::new(ctor));
        }
        self
    }

    pub fn with_handler(mut self, handler: impl Handler) -> Self {
        self.handler = Some(Arc::new(handler));
        self
    }

    pub fn with_gate(mut self, gate: impl Fn(&Data) -> bool + Send + Sync + 'static) -> Self {
        self.gate = Some(Box::new(gate));
        self
    }

    pub fn requires_ai(mut self) -> Self {
        self.needs_ai = true;
        self
    }
}

impl Feature for FeatureBuilder {
    fn needs_ai(&self) -> bool {
        self.needs_ai
    }
    fn enabled(&self, data: &Data) -> bool {
        self.gate.as_ref().is_none_or(|g| g(data))
    }
    fn commands(&self, data: &Data) -> Vec<Command<Data, Error>> {
        let reg = &data.config.commands_registry;
        self.commands
            .iter()
            .map(|make| make().decorate(reg))
            .collect()
    }
    fn handler(&self) -> Option<Arc<dyn Handler>> {
        self.handler.clone()
    }
}
