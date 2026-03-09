use disaity::core::{core, utils::load_bin};

#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt::init();

    load_bin();

    core().await.unwrap();
}
