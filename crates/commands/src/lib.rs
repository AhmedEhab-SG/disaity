mod chat;
mod checks;
mod music;
mod other;
mod utils;

use std::{collections::HashSet, str::FromStr};

use poise::Command as PoiseCommand;

use disaity_config::{Category, CommandId};
use disaity_core::{Data, Decorate, Error, Feature};

use chat::ask;
use music::{clear, jump, pause, play, queue, repeat, resume, seek, shuffle, skip, stop, volume};
use other::{help, join, leave};
use strum::VariantArray;

#[derive(Default)]
pub struct Commands {
    picks: Vec<CommandId>,
    categories: Vec<Category>,
    needs_ai: bool,
    needs_db: bool,
}

impl Commands {
    pub fn new() -> Self {
        Self::default()
    }

    fn ctor(cmd: &CommandId) -> Option<fn() -> PoiseCommand<Data, Error>> {
        Some(match cmd {
            CommandId::Ask => ask,
            CommandId::Help => help,
            CommandId::Join => join,
            CommandId::Leave => leave,
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

            CommandId::Prayer | CommandId::ClearPrayer => return None,
        })
    }

    pub fn with(mut self, cmd: CommandId) -> Self {
        self.picks.push(cmd);
        self
    }

    pub fn with_all(mut self, cmds: impl IntoIterator<Item = CommandId>) -> Self {
        self.picks.extend(cmds);
        self
    }

    pub fn with_category(mut self, cat: Category) -> Self {
        self.categories.push(cat);
        self
    }

    pub fn requires_ai(mut self) -> Self {
        self.needs_ai = true;
        self
    }

    pub fn requires_db(mut self) -> Self {
        self.needs_db = true;
        self
    }

    pub fn music() -> Self {
        Self::new().with_category(Category::Music)
    }

    pub fn chat() -> Self {
        Self::new().with_category(Category::Chat).requires_ai()
    }

    pub fn other() -> Self {
        Self::new().with_category(Category::Other)
    }

    pub fn with_all_categories(mut self) -> Self {
        self.categories.extend(Category::VARIANTS.iter().cloned());
        self
    }

    pub fn all() -> Self {
        Self::new().with_all_categories().requires_ai()
    }
}

impl Feature for Commands {
    fn needs_ai(&self) -> bool {
        self.needs_ai
    }

    fn needs_db(&self) -> bool {
        self.needs_db
    }

    fn commands(&self, data: &Data) -> Vec<PoiseCommand<Data, Error>> {
        let reg = &data.config.commands_registry;
        let mut wanted: Vec<CommandId> = Vec::new();

        for cat in &self.categories {
            for config in reg.get_cmds_from_cat(cat) {
                match CommandId::from_str(&config.name) {
                    Ok(cmd) => wanted.push(cmd),
                    Err(_) => eprintln!(
                        "Warning: command '{}' in category '{cat}' has no enum variant",
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
