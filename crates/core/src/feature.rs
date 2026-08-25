use std::{str::FromStr, sync::Arc};

use disaity_config::{Command as CommandEnum, CommandRegistry};
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

pub fn decorate(mut cmd: Command<Data, Error>, registry: &CommandRegistry) -> Command<Data, Error> {
    let cmd_enum =
        CommandEnum::from_str(cmd.name.as_str()).expect("command missing from registry enum");
    let config = registry.get_command(&cmd_enum);

    cmd.name = config.name.clone();
    cmd.description = Some(config.description.clone());
    cmd.aliases = config.keys.clone();
    cmd.category = Some(config.category.clone());
    cmd
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
        vec![decorate(self.0.add(), reg), decorate(self.0.clear(), reg)]
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
        self.gate.as_ref().map_or(true, |g| g(data))
    }
    fn commands(&self, data: &Data) -> Vec<Command<Data, Error>> {
        let reg = &data.config.commands_registry;
        self.commands
            .iter()
            .map(|make| decorate(make(), reg))
            .collect()
    }
    fn handler(&self) -> Option<Arc<dyn Handler>> {
        self.handler.clone()
    }
}
