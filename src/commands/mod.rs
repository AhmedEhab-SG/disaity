mod chat;
mod checks;
mod music;
mod other;
mod subscription;

use chat::ask;
use music::{clear, jump, pause, play, queue, repeat, resume, seek, shuffle, skip, stop, volume};
use other::{help, join, leave};

pub use subscription::Subscription;
use subscription::{clear_prayer, prayer};

use poise::Command;
use std::str::FromStr;

use crate::{
    config::{Command as CommandEnum, CommandRegistry},
    core::{Data, Error},
};

#[derive(Debug)]
pub struct CommandsRegistry {
    pub commands: Vec<Command<Data, Error>>,
    pub subscription: Subscription,
}

impl CommandsRegistry {
    pub fn new(cmds_registery: &CommandRegistry, db_path: &String) -> Self {
        let commands = vec![
            ask(),
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
            help(),
            join(),
            leave(),
            clear_prayer(),
            prayer(),
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

        let subscription = Subscription::new(db_path);

        Self {
            commands,
            subscription,
        }
    }
}
