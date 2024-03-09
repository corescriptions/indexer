use rocksdb::DB;
use serde_json::json;
use std::sync::{Arc, RwLock};

pub mod router_inscription;
pub mod router_market;
pub mod router_nft;
pub mod router_other;
pub mod router_token;
pub mod server;

pub const PAGE_SIZE: u64 = 16;

type WebData = actix_web::web::Data<std::sync::Arc<APIState>>;

pub struct APIState {
    pub db: Arc<RwLock<DB>>,
    pub blocknumber: RwLock<u64>,
}

pub struct DBRefresh;

pub trait HttpResponseExt {
    fn response_data<T: serde::Serialize>(value: T) -> Self;
    fn response_error(error_code: u64, error: &str) -> Self;
    fn response_error_notfound() -> Self;
}

impl HttpResponseExt for actix_web::HttpResponse {
    fn response_data<T: serde::Serialize>(value: T) -> Self {
        Self::Ok().json(json!({ "error_code": 0, "data": value }))
    }

    fn response_error(error_code: u64, error: &str) -> Self {
        Self::Ok().json(json!({ "error_code": error_code, "error": error }))
    }

    fn response_error_notfound() -> Self {
        Self::Ok().json(json!({ "error_code": 404, "error": "not found" }))
    }
}
