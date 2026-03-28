pub mod utils;

use ::serenity::all::GatewayIntents;
use gemini_rust::Gemini;
use poise::{
    Context as BaseContext, Framework, FrameworkOptions, PrefixFrameworkOptions, builtins,
    serenity_prelude as serenity,
};

use songbird::SerenityInit;

use crate::{
    commands::{
        chat::ask::ask,
        music::{
            clear::clear, jump::jump, pause::pause, play::play, queue::queue, repeat::repeat,
            resume::resume, seek::seek, shuffle::shuffle, skip::skip, stop::stop,
        },
        others::{help::help, join::join, leave::leave},
    },
    config::commands::CommandRegistry,
    handlers::ready::start_status_loop,
};

pub struct Data {
    pub http: reqwest::Client,
    pub agent: Gemini,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = BaseContext<'a, Data, Error>;

pub async fn core() -> Result<(), Error> {
    let token = dotenv::var("CLIENT_TOKEN")?;
    let ai_key = dotenv::var("GEMINI_API_KEY")?;

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_INTEGRATIONS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let agent = Gemini::new(ai_key)?;
    let http = reqwest::Client::new();
    let cmds_registery = CommandRegistry::new();

    let mut commands = vec![
        play(),
        pause(),
        stop(),
        skip(),
        clear(),
        resume(),
        help(),
        join(),
        leave(),
        jump(),
        repeat(),
        queue(),
        shuffle(),
        seek(),
        ask(),
    ];

    for cmd in &mut commands {
        if let Some(config) = cmds_registery.commands.get(&cmd.name) {
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
        }
    }

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands,
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("-".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                builtins::register_globally(ctx, &framework.options().commands).await?;
                start_status_loop(ctx.clone());
                Ok(Data { http, agent })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .await;

    client?.start().await?;

    Ok(())
}
