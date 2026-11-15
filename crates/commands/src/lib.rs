mod chat;
mod checks;
mod guard;
mod manage;
mod music;
mod other;
mod utils;

use std::{collections::HashSet, str::FromStr};

use poise::Command as PoiseCommand;
use strum::VariantArray;

use disaity_config::{Category, CommandId};
use disaity_core::{Client, Data, Decorate, Error, Feature};

pub use checks::Check;
pub use guard::Guard;
pub use utils::Utils;

use chat::{ask, assist};
use manage::{kick, move_member, role};
use music::{clear, jump, pause, play, queue, repeat, resume, seek, shuffle, skip, stop, volume};
use other::{help, join, leave, ping};

#[derive(Default)]
pub struct Commands {
    picks: Vec<CommandId>,
    categories: Vec<Category>,
}

impl Commands {
    pub fn new() -> Self {
        Self::default()
    }

    fn ctor(cmd: &CommandId) -> Option<fn() -> PoiseCommand<Data, Error>> {
        Some(match cmd {
            // music
            CommandId::Clear => clear,
            CommandId::Jump => jump,
            CommandId::Pause => pause,
            CommandId::Play => play,
            CommandId::Queue => queue,
            CommandId::Repeat => repeat,
            CommandId::Resume => resume,
            CommandId::Seek => seek,
            CommandId::Shuffle => shuffle,
            CommandId::Skip => skip,
            CommandId::Stop => stop,
            CommandId::Volume => volume,

            // subscription (ignored bec its a feature needs other helpers)
            CommandId::FreeGames => return None,

            // manage
            CommandId::Kick => kick,
            CommandId::MoveMember => move_member,
            CommandId::Role => role,

            // chat
            CommandId::Ask => ask,
            CommandId::Assist => assist,

            // others
            CommandId::Help => help,
            CommandId::Join => join,
            CommandId::Leave => leave,
            CommandId::Ping => ping,
        })
    }

    pub fn with_id(mut self, cmd: CommandId) -> Self {
        self.picks.push(cmd);
        self
    }

    pub fn with_ids(mut self, cmds: impl IntoIterator<Item = CommandId>) -> Self {
        self.picks.extend(cmds);
        self
    }

    pub fn with_category(mut self, cat: Category) -> Self {
        self.categories.push(cat);
        self
    }

    pub fn with_categories(mut self, cats: impl IntoIterator<Item = Category>) -> Self {
        self.categories.extend(cats);
        self
    }

    pub fn all() -> Self {
        Self::new().with_categories(Category::VARIANTS.iter().cloned())
    }
}

impl Feature for Commands {
    fn commands(&self, data: &Data) -> Vec<PoiseCommand<Data, Error>> {
        let reg = &data.config.commands_registry;
        let mut wanted: Vec<CommandId> = Vec::new();

        for cat in &self.categories {
            for config in reg.get_cmds_from_cat(cat) {
                match CommandId::from_str(&config.name) {
                    Ok(cmd) => wanted.push(cmd),
                    Err(_) => tracing::warn!(
                        "command '{}' in category '{cat}' has no enum variant, skipping it",
                        config.name
                    ),
                }
            }
        }

        wanted.extend(self.picks.iter().cloned());

        let mut seen = HashSet::new();

        wanted
            .into_iter()
            .filter(|cmd| seen.insert(cmd.clone()))
            .filter_map(|cmd| Self::ctor(&cmd))
            .map(|make| make().decorate(reg))
            .collect()
    }
}

pub trait CommandsExt {
    fn with_command(self, id: CommandId) -> Self;
    fn with_commands(self, commands: Commands) -> Self;
    fn with_raw_commands(
        self,
        commands: impl IntoIterator<Item = PoiseCommand<Data, Error>>,
    ) -> Self;
}

impl CommandsExt for Client {
    fn with_raw_commands(
        mut self,
        commands: impl IntoIterator<Item = PoiseCommand<Data, Error>>,
    ) -> Self {
        self.commands.extend(commands);
        self
    }

    fn with_command(self, id: CommandId) -> Self {
        self.with_commands(Commands::new().with_id(id))
    }

    fn with_commands(self, commands: Commands) -> Self {
        self.with_feature(commands)
    }
}
