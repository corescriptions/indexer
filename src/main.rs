use dotenv::dotenv;
use insdexer::{
    adjust_open_files, api,
    config::{self, API_ONLY},
    inscription,
};
use log::info;
use tokio;

#[tokio::main]
async fn main() {
    dotenv().ok();

    log4rs::init_file("./log4rs.yaml", Default::default()).unwrap();

    info!("{:?}", *config::ARGS);

    adjust_open_files::adjust_open_files_limit();

    ctrlc::set_handler(|| {
        info!("Received Ctrl+C signal. Exiting...");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let indexer = inscription::types::Indexer::new();
    indexer.init();

    api::server::run(*API_ONLY).await;

    indexer.run().await;
}
