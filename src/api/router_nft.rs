use super::{router_inscription::insc_list_to_display, HttpResponseExt, WebData, PAGE_SIZE};
use crate::{
    inscription::{db::*, types::*},
    num_index,
};
use actix_web::{get, web, web::Query, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine as _};
use rocksdb::Direction;
use serde::{Deserialize, Serialize};

pub fn register(config: &mut web::ServiceConfig) {
    config.service(nft_recent);
    config.service(nft_collections);
    config.service(nfts);
    config.service(nft);
    config.service(nft_transfers);
}

#[derive(Debug, Serialize, Deserialize)]
struct RecentNFTsParams {
    page: Option<u64>,
    include_image_data: Option<bool>,
}

#[get("/nft_recent")]
async fn nft_recent(info: Query<RecentNFTsParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let key_list = db.get_item_keys(
        KEY_INSC_INDEX_NFT_ID,
        KEY_INSC_INDEX_NFT_ID,
        page * PAGE_SIZE,
        PAGE_SIZE,
        Direction::Forward,
    );
    let id_list = db_index2id_desc(key_list);
    let insc_list = db.get_inscriptions_by_id(&id_list);

    HttpResponse::response_data(insc_list_to_display(&db, &insc_list))
}

#[derive(Debug, Serialize, Deserialize)]
struct NFTTransfersParams {
    page: Option<u64>,
    tx: Option<String>,
    id: Option<u64>,
}

#[get("/nft_transfers")]
async fn nft_transfers(info: Query<NFTTransfersParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let nft_insc = if let Some(insc_id) = info.id {
        db.get_inscription_by_id(insc_id)
    } else if let Some(insc_tx) = &info.tx {
        db.get_inscription_by_tx(insc_tx)
    } else {
        None
    };

    let nft_insc = match nft_insc {
        Some(value) => value,
        None => return HttpResponse::response_error_notfound(),
    };

    let page = info.page.unwrap_or(1) - 1;
    let prefix = make_index_key(KEY_INSC_NFT_TRANS_INDEX_ID, num_index!(nft_insc.id)) + ":";
    let key_list = db.get_item_keys(&prefix, &prefix, page * PAGE_SIZE, PAGE_SIZE, Direction::Forward);
    let transfer_id_list = db_index2id_desc(key_list);
    let transfer_list = db.get_inscriptions_by_id(&transfer_id_list);

    HttpResponse::response_data(insc_list_to_display(&db, &transfer_list))
}

#[derive(Debug, Serialize, Deserialize)]
struct CollectionsParams {
    page: Option<u64>,
}

#[get("/nft_collections")]
async fn nft_collections(info: Query<CollectionsParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let key_list = db.get_item_keys(
        KEY_INSC_NFT_COLL_INDEX_ID,
        KEY_INSC_NFT_COLL_INDEX_ID,
        page * PAGE_SIZE,
        PAGE_SIZE,
        Direction::Forward,
    );
    let id_list = db_index2id_desc(key_list);
    let insc_list = db.get_inscriptions_by_id(&id_list);

    HttpResponse::response_data(insc_list)
}

#[derive(Debug, Serialize, Deserialize)]
struct NFTsParams {
    page: Option<u64>,
    address: String,
    include_image_data: Option<bool>,
}

#[get("/nfts")]
async fn nfts(info: Query<NFTsParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let prefix = make_index_key(KEY_INSC_NFT_INDEX_HOLDER_ADDRESS, &info.address.to_lowercase());
    let key_list = db.get_item_keys(&prefix, &prefix, page * PAGE_SIZE, PAGE_SIZE, Direction::Forward);
    let id_list = db_index2id_desc(key_list);
    let insc_list = db.get_inscriptions_by_id(&id_list);

    HttpResponse::response_data(insc_list_to_display(&db, &insc_list))
}

#[get("/nft/{path}")]
async fn nft(path: web::Path<String>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let path_value = path.into_inner();
    let result = match path_value.parse::<u64>() {
        Ok(id) => db.get_inscription_by_id(id),
        Err(_) => db.get_inscription_by_tx(&path_value),
    };

    match result {
        Some(insc) => match insc.mime_category {
            InscriptionMimeCategory::Image => {
                let bytes = general_purpose::STANDARD.decode(&insc.mime_data).unwrap();
                HttpResponse::Ok().content_type(insc.mime_type).body(bytes)
            }
            InscriptionMimeCategory::Text => HttpResponse::Ok()
                .content_type("text/plain; charset=utf-8")
                .body(insc.mime_data),
            _ => HttpResponse::BadRequest().body("Not an NFT"),
        },
        None => HttpResponse::NotFound().body("Inscription not found"),
    }
}
