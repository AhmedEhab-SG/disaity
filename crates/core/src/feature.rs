use std::{str::FromStr, sync::Arc};

use disaity_config::{AiAccess, CommandId, CommandRegistry};
use poise::{Command, command};

use crate::{CommandMeta, Context, Data, Error, Handler};

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
        let Ok(id) = CommandId::from_str(&self.name) else {
            return self;
        };

        // the registry's text and ai access, written onto one command
        let apply = |cmd: &mut Command<Data, Error>,
                     name: &str,
                     keys: &[String],
                     description: &str,
                     ai: Option<AiAccess>| {
            cmd.name = name.to_owned().into();
            cmd.description = Some(description.to_owned().into());
            cmd.aliases = keys.iter().cloned().map(Into::into).collect();

            // apply custom data with disaity meta data
            if let Some(access) = ai {
                CommandMeta::in_slot(&mut cmd.custom_data).ai = access;
            }
        };

        let entry = registry.get_command(&id);

        apply(
            &mut self,
            &entry.name,
            &entry.keys,
            &entry.description,
            entry.ai,
        );
        self.category = Some(entry.category.clone().into());

        // a parent's subcommands read the same entry, keyed by their function name
        for sub in &mut self.subcommands {
            let sub_entry = entry.subcommand(&sub.name);

            apply(
                sub,
                &sub_entry.name,
                &sub_entry.keys,
                &sub_entry.description,
                sub_entry.ai,
            );
            sub.category = self.category.clone();
        }

        self
    }
}

/// a background feed a server signs up for. disaity builds the parent command,
/// the three actions under it are yours: `/<name> set`, `clear` and `status`
pub trait SubscriptionModule: Send + Sync + 'static {
    /// the parent command's name, `cmd_name` in `/cmd_name set`
    fn name(&self) -> &'static str;
    fn set(&self) -> Command<Data, Error>;
    fn clear(&self) -> Command<Data, Error>;
    fn status(&self) -> Command<Data, Error>;
    fn handler(&self) -> Arc<dyn Handler>;
    fn enabled(&self, _data: &Data) -> bool {
        true
    }
}

pub struct AsSubscription<S: SubscriptionModule>(pub S);

impl<S: SubscriptionModule> AsSubscription<S> {
    // the parent every subscription gets. it never runs: slash can't call it alone,
    // and a bare prefix call gets poise's "pick a subcommand" reply.
    // the doc line is the slash description of subscriptions that aren't in commands.json
    /// Set up, clear or check this subscription.
    #[command(slash_command, prefix_command, guild_only, category = "subscription")]
    async fn subscription(_ctx: Context<'_>) -> Result<(), Error> {
        Ok(())
    }
}

impl<S: SubscriptionModule> Feature for AsSubscription<S> {
    fn enabled(&self, data: &Data) -> bool {
        self.0.enabled(data)
    }

    fn needs_db(&self) -> bool {
        true
    }

    fn commands(&self, data: &Data) -> Vec<Command<Data, Error>> {
        let mut parent = Self::subscription();

        parent.name = self.0.name().into();

        parent.subcommands = vec![self.0.set(), self.0.clear(), self.0.status()];

        parent.subcommand_required = true;

        vec![parent.decorate(&data.config.commands_registry)]
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
