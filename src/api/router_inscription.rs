use super::{router_market::market_order_to_display, HttpResponseExt, WebData, PAGE_SIZE};
use crate::{
    inscription::{db::*, marketplace::db::InscribeMarketDB, types::*},
    num_index,
};
use actix_web::{get, web, web::Query, HttpResponse, Responder};
use rocksdb::Direction;
use serde::{Deserialize, Serialize};

pub fn register(config: &mut web::ServiceConfig) {
    config.service(inscription);
    config.service(inscription_path);
    config.service(recent);
    config.service(transactions);
    config.service(created);
}

pub fn inscription_to_display(db: &rocksdb::DB, insc: &Inscription, include_imagedata: bool) -> serde_json::Value {
    let mut insc_json = serde_json::to_value(insc).unwrap();
    if let Some(_) = &insc.signature {
        let holder = db.get_inscription_nft_holder_by_id(insc.id);
        insc_json["owner"] = serde_json::to_value(holder).unwrap();

        if let Some(collection) = db.get_inscription_nft_collection_by_id(insc.id) {
            insc_json["collection"] = serde_json::to_value(collection).unwrap();
        }
    }

    if let Some(market_order_id) = &insc.market_order_id {
        if let Some(market_order) = db.market_get_order_by_id(market_order_id) {
            insc_json["market_order_info"] = market_order_to_display(&market_order);
        }
    }

    if !include_imagedata && insc.mime_category == InscriptionMimeCategory::Image {
        insc_json["mime_data"] = serde_json::to_value("").unwrap();
    }

    insc_json["mime_category"] = serde_json::to_value(match insc.mime_category {
        InscriptionMimeCategory::Null => "null",
        InscriptionMimeCategory::Text => "text",
        InscriptionMimeCategory::Image => "image",
        InscriptionMimeCategory::Transfer => "transfer",
        InscriptionMimeCategory::Json => "json",
        InscriptionMimeCategory::Invoke => "invoke",
    })
    .unwrap();

    insc_json["verified"] = serde_json::to_value(match insc.verified {
        InscriptionVerifiedStatus::Unresolved => "unknown",
        InscriptionVerifiedStatus::Successful => "success",
        InscriptionVerifiedStatus::Failed => "fail",
    })
    .unwrap();

    insc_json
}

pub fn insc_list_to_display(db: &rocksdb::DB, insc_list: &Vec<Inscription>) -> Vec<serde_json::Value> {
    let mut result = vec![];
    for insc in insc_list {
        result.push(inscription_to_display(db, insc, false));
    }
    result
}

#[derive(Debug, Serialize, Deserialize)]
struct InscriptionsParams {
    id: Option<u64>,
    tx: Option<String>,
}

#[get("/inscription")]
async fn inscription(query: Query<InscriptionsParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let result = if let Some(insc_id) = query.id {
        db.get_inscription_by_id(insc_id)
    } else if let Some(insc_tx) = &query.tx {
        db.get_inscription_by_tx(insc_tx)
    } else {
        None
    };

    if let Some(insc) = result {
        HttpResponse::response_data(inscription_to_display(&db, &insc, true))
    } else {
        HttpResponse::response_error_notfound()
    }
}

#[get("/inscription/{path}")]
async fn inscription_path(path: web::Path<String>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let path_value = path.into_inner();
    let result = match path_value.parse::<u64>() {
        Ok(id) => db.get_inscription_by_id(id),
        Err(_) => db.get_inscription_by_tx(&path_value),
    };

    if let Some(insc) = result {
        HttpResponse::response_data(inscription_to_display(&db, &insc, true))
    } else {
        HttpResponse::response_error_notfound()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RecentParams {
    page: Option<u64>,
}

#[get("/recent")]
async fn recent(info: Query<RecentParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let top_insc_id = db.get_top_inscription_id();
    let start_key = make_index_key(KEY_INSC_INDEX_ID, num_index!(top_insc_id));
    let insc_list = db.get_items(KEY_INSC_INDEX_ID, &start_key, page * PAGE_SIZE, PAGE_SIZE, Direction::Reverse);
    let insc_list = db_item_val2json::<Inscription>(insc_list);

    HttpResponse::response_data(insc_list_to_display(&db, &insc_list))
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionsParams {
    page: Option<u64>,
    address: String,
}

#[get("/transactions")]
async fn transactions(info: Query<TransactionsParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let prefix = make_index_key(KEY_INSC_INDEX_ADDRESS, &info.address.to_lowercase());
    let key_list = db.get_item_keys(&prefix, &prefix, page * PAGE_SIZE, PAGE_SIZE, Direction::Forward);
    let id_list = db_index2id_desc(key_list);
    let insc_list = db.get_inscriptions_by_id(&id_list);

    HttpResponse::response_data(insc_list_to_display(&db, &insc_list))
}

#[derive(Debug, Serialize, Deserialize)]
struct CreatedParams {
    page: Option<u64>,
    address: String,
}

#[get("/created")]
async fn created(info: Query<CreatedParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let prefix = make_index_key(KEY_INSC_INDEX_CREATER, &info.address.to_lowercase());
    let key_list = db.get_item_keys(&prefix, &prefix, page * PAGE_SIZE, PAGE_SIZE, Direction::Forward);
    let id_list = db_index2id_desc(key_list);
    let insc_list = db.get_inscriptions_by_id(&id_list);

    HttpResponse::response_data(insc_list_to_display(&db, &insc_list))
}
