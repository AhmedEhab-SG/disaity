pub mod chat;
pub mod checks;
pub mod music;
pub mod other;
pub mod subscription;

use poise::Command;
use std::str::FromStr;

use crate::{
    config::commands::{Command as CommandEnum, CommandRegistry},
    core::{context::Data, errors::Error},
};

#[derive(Debug)]
pub struct CommandsRegistry {
    pub commands: Vec<Command<Data, Error>>,
}

impl CommandsRegistry {
    pub fn new(cmds_registery: &CommandRegistry) -> Self {
        let commands = vec![
            music::play::play(),
            music::pause::pause(),
            music::stop::stop(),
            music::skip::skip(),
            music::clear::clear(),
            music::resume::resume(),
            music::jump::jump(),
            music::repeat::repeat(),
            music::queue::queue(),
            music::shuffle::shuffle(),
            music::seek::seek(),
            music::volume::volume(),
            chat::ask::ask(),
            other::help::help(),
            other::join::join(),
            other::leave::leave(),
            subscription::prayer_manager::prayer::prayer(),
            subscription::prayer_manager::clear_prayer::clear_prayer(),
        ]
        .into_iter()
        .map(|mut cmd| {
            let cmd_enum =
                CommandEnum::from_str(cmd.name.as_str()).expect("Failed to get command name");

            let config = cmds_registery.get_command(&cmd_enum);

            cmd.name = config.name.clone();
            cmd.description = Some(config.description.clone());
            cmd.aliases = config.keys.clone();
            cmd.category = Some(config.category.clone());

            // if let Some(json_options) = &config.options {
            //     for (param, json_opt) in cmd.parameters.iter_mut().zip(json_options.iter()) {
            //         param.name = json_opt.name.clone();
            //         param.description = Some(json_opt.description.clone());
            //         param.required = json_opt.required;
            //
            //         if let Some(json_choices) = &json_opt.choices {
            //             let mut poise_choices = Vec::new();
            //
            //             for choice in json_choices {
            //                 poise_choices.push(poise::CommandParameterChoice {
            //                     name: choice.name.clone(),
            //                     localizations: Default::default(),
            //                     __non_exhaustive: (),
            //                 });
            //             }
            //             param.choices = poise_choices;
            //             param.required = json_opt.required;
            //         }
            //     }
            // }
            cmd
        })
        .collect();

        Self { commands }
    }
}
