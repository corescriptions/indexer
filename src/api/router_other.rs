use super::{HttpResponseExt, WebData};
use crate::{
    global::ROLLBACK_BLOCK,
    inscription::{db::*, db_checkpoint::checkpoints_list},
};
use actix_web::{
    get,
    web::{self, Query},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn register(config: &mut web::ServiceConfig) {
    config.service(status);
    config.service(checkpoints);
    config.service(rollback);
}

#[get("/status")]
async fn status(state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let top_insc_id = db.get_top_inscription_id();
    let top_sync_id = db.get_top_inscription_sync_id();
    let sync_blocknumber = db.get_sync_blocknumber();

    match db.get_inscription_by_id(top_insc_id) {
        Some(insc) => {
            let current_blocknumber = *state.blocknumber.read().unwrap();
            let insc_blocknumber = insc.blocknumber;
            let result = json!({
                "latest_id": top_insc_id,
                "latest_sync_id": top_sync_id,
                "block_number": current_blocknumber,
                "block_number_inscribe": insc_blocknumber,
                "block_number_sync": sync_blocknumber,
                "block_number_behind": insc_blocknumber as i64 - current_blocknumber as i64,
            });
            HttpResponse::response_data(result)
        }
        None => HttpResponse::response_error_notfound(),
    }
}

#[get("/checkpoints")]
async fn checkpoints() -> impl Responder {
    HttpResponse::response_data(checkpoints_list())
}

#[derive(Debug, Serialize, Deserialize)]
struct RollbackParams {
    blocknumber: u64,
}

#[get("/rollback")]
async fn rollback(info: Query<RollbackParams>) -> impl Responder {
    if *ROLLBACK_BLOCK.lock().unwrap() == 0 {
        *ROLLBACK_BLOCK.lock().unwrap() = info.blocknumber;
        HttpResponse::response_data("ok")
    } else {
        HttpResponse::response_error(1, "rollback in progress")
    }
}
