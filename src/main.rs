use disaity::core::{bin::BinariesExt, core};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    BinariesExt::load();

    core().await.unwrap();
}
