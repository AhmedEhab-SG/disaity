use disaity_commands::CommandFactory;
use disaity_core::{BinariesExt, Client, DataBuilder, Error};
use disaity_handlers::{PrayerHandler, StatusHandler};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    BinariesExt::load();

    let data = DataBuilder::new().build();
    let token = data.config.env.client_token.clone();
    let prefix = data.config.info_registry.prefix.clone();
    let cmd_factory = CommandFactory::new(&data.config.commands_registry);

    Client::new(token)
        .with_prefix(prefix)
        .with_commands(cmd_factory.all())
        .with_handler(StatusHandler)
        .with_handler(PrayerHandler)
        .with_data(data)
        .run()
        .await
}
