use disaity::core::{bin, core};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    bin::load();

    core().await.unwrap();
}
