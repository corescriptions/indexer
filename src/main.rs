use dotenv::dotenv;
use insdexer::{adjust_open_files, api, config, inscription};
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

    let mut indexer = inscription::types::Indexer::new();

    api::server::run().await;

    indexer.init();
    indexer.run().await;
}
