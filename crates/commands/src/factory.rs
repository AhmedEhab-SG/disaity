use std::str::FromStr;

use poise::Command;
use strum::VariantArray;

use disaity_config::{Category, Command as CommandEnum, CommandRegistry};
use disaity_core::{Data, Error};

use super::chat::ask;
use super::music::{
    clear, jump, pause, play, queue, repeat, resume, seek, shuffle, skip, stop, volume,
};
use super::other::{help, join, leave};
use super::subscription::{clear_prayer, prayer};

pub struct CommandFactory<'a> {
    registry: &'a CommandRegistry,
}

impl<'a> CommandFactory<'a> {
    pub fn new(registry: &'a CommandRegistry) -> Self {
        Self { registry }
    }

    fn apply(&self, mut cmd: Command<Data, Error>) -> Command<Data, Error> {
        let cmd_enum =
            CommandEnum::from_str(cmd.name.as_str()).expect("Failed to get command name");
        let config = self.registry.get_command(&cmd_enum);

        cmd.name = config.name.clone();
        cmd.description = Some(config.description.clone());
        cmd.aliases = config.keys.clone();
        cmd.category = Some(config.category.clone());
        cmd
    }

    fn build(&self, cmds: Vec<Command<Data, Error>>) -> Vec<Command<Data, Error>> {
        cmds.into_iter().map(|c| self.apply(c)).collect()
    }

    pub fn category(&self, cat: &Category) -> Vec<Command<Data, Error>> {
        let cmds = match cat {
            Category::Music => vec![
                clear(),
                jump(),
                pause(),
                play(),
                queue(),
                repeat(),
                resume(),
                seek(),
                shuffle(),
                skip(),
                stop(),
                volume(),
            ],
            Category::Subscription => vec![prayer(), clear_prayer()],
            Category::Chat => vec![ask()],
            Category::Other => vec![help(), join(), leave()],
        };
        self.build(cmds)
    }

    pub fn all(&self) -> Vec<Command<Data, Error>> {
        Category::VARIANTS
            .iter()
            .flat_map(|cat| self.category(cat))
            .collect()
    }
}
