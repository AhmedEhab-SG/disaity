use disaity_commands::CommandFactory;
use disaity_config::{ConfigBuilder, EnvBuilder, Info, Persona, Preset};
use disaity_core::{BinariesExt, Client, DataBuilder, Error};
use disaity_handlers::{PrayerHandler, StatusHandler};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    BinariesExt::load();

    let data = DataBuilder::new()
        .with_config(
            ConfigBuilder::new()
                .with_persona(Persona::new(Preset::Emilia))
                .build(),
        )
        .build()
        .await?;

    let client = Client::new(&data.config.env.client_token)
        .with_prefix(&data.config.info.prefix)
        .with_commands(CommandFactory::new(&data.config.commands_registry).all())
        .with_handler(StatusHandler)
        .with_handler(PrayerHandler)
        .with_data(data);

    let data_alt = DataBuilder::new()
        .with_config(
            ConfigBuilder::new()
                .with_env(
                    EnvBuilder::new()
                        .with_client_token("CLIENT_TOKEN_ALT")
                        .build(),
                )
                .with_info(Info::new().with_prefix("~"))
                .with_persona(Persona::new(Preset::Rem))
                .build(),
        )
        .build()
        .await?;

    let client_alt = Client::new(&data_alt.config.env.client_token)
        .with_prefix(&data_alt.config.info.prefix)
        .with_commands(CommandFactory::new(&data_alt.config.commands_registry).all())
        .with_handler(StatusHandler)
        .with_data(data_alt);

    let handle_emilia = tokio::spawn(client.run());
    let handle_rem = tokio::spawn(client_alt.run());

    let (thread_emilia, thread_rem) = tokio::try_join!(handle_emilia, handle_rem)?;
    thread_emilia?;
    thread_rem?;

    Ok(())
}
