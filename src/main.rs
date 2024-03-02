use dotenv::dotenv;
use insdexer::{api, config::OPEN_FILES_LIMIT, inscription};
use log::info;
use tokio;

fn adjust_open_files_limit() {
    let limit = *OPEN_FILES_LIMIT;
    if limit == 0 {
        return;
    }

    let mut rlimit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlimit) };
    println!("current open files limit: {}", rlimit.rlim_cur);

    let new_limit = libc::rlimit {
        rlim_cur: limit,
        rlim_max: limit,
    };
    unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &new_limit) };

    unsafe {
        libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlimit);
    }
    println!("new open files limit: {}", rlimit.rlim_cur);
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    log4rs::init_file("./log4rs.yaml", Default::default()).unwrap();

    adjust_open_files_limit();

    ctrlc::set_handler(|| {
        info!("Received Ctrl+C signal. Exiting...");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let mut indexer = inscription::types::Indexer::new();

    api::server::run(indexer.db.clone()).await;

    indexer.init();
    indexer.run().await;
}
