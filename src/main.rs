use disaity::core::{BinariesExt, create};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    BinariesExt::load();

    create().await.unwrap();
}
