use super::{APIState, WebData};
use crate::{
    config::{DB_PATH, HTTP_BIND, HTTP_PORT, WEB3_PROVIDER},
    ethereum::{init_web3_http, Web3Ex},
    global::sleep_ms,
};
use actix_cors::Cors;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use log::{info, warn};
use std::sync::{Arc, RwLock};

impl APIState {
    pub fn new() -> Self {
        Self {
            db: Arc::new(RwLock::new(Self::db())),
            blocknumber: 0.into(),
        }
    }

    pub fn db() -> rocksdb::DB {
        let options = rocksdb::Options::default();
        loop {
            return match rocksdb::DB::open_for_read_only(&options, DB_PATH.as_str(), false) {
                Ok(db) => db,
                Err(e) => {
                    warn!("[api] open db failed: {}, retry", e);
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    continue;
                }
            };
        }
    }
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("CoreScriptions Insdexer RESTful API Server")
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("404 Not Found")
}

async fn blocknumber_refresh(state: WebData) {
    let web3 = init_web3_http(WEB3_PROVIDER.to_string().as_str());
    loop {
        let blocknumber = web3.get_blocknumber_wait().await;
        *state.blocknumber.write().unwrap() = blocknumber;
        sleep_ms(3000).await;
    }
}

async fn db_refresh(state: WebData) {
    loop {
        let db = APIState::db();
        *state.db.write().unwrap() = db;
        sleep_ms(3000).await;
    }
}

pub async fn run(wait_forever: bool) {
    let state = web::Data::new(Arc::new(APIState::new()));
    tokio::spawn(blocknumber_refresh(state.clone()));
    tokio::spawn(db_refresh(state.clone()));

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .wrap(Cors::default().allow_any_origin().allow_any_method().allow_any_header())
            .default_service(web::route().to(not_found))
            .service(index)
            .configure(super::router_inscription::register)
            .configure(super::router_market::register)
            .configure(super::router_nft::register)
            .configure(super::router_token::register)
            .configure(super::router_other::register)
    })
    .bind((HTTP_BIND.as_str(), *HTTP_PORT))
    .unwrap()
    .run();

    info!("RESTful API server started at http://{}:{}", *HTTP_BIND, *HTTP_PORT);

    if wait_forever {
        info!("Running on API_ONLY mode");
        server.await.unwrap();
    } else {
        tokio::spawn(server);
    }
}
