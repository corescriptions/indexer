use super::{HttpResponseExt, WebData, PAGE_SIZE};
use crate::inscription::{db::*, types::*};
use actix_web::{get, web, web::Query, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine as _};
use rocksdb::Direction;
use serde::{Deserialize, Serialize};

pub fn register(config: &mut web::ServiceConfig) {
    config.service(recent_nfts);
    config.service(collections);
    config.service(nfts);
    config.service(nft);
}

#[derive(Debug, Serialize, Deserialize)]
struct RecentNFTsParams {
    page: Option<u64>,
    include_image_data: Option<bool>,
}

#[get("/nft_recent")]
async fn recent_nfts(info: Query<RecentNFTsParams>, state: WebData) -> impl Responder {
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
    let mut insc_list = db.get_inscriptions_by_id(&id_list);

    if !info.include_image_data.unwrap_or(false) {
        for insc in insc_list.iter_mut() {
            if insc.mime_category == InscriptionMimeCategory::Image {
                insc.mime_data = "".to_string();
            }
        }
    }

    HttpResponse::response_data(insc_list)
}

#[derive(Debug, Serialize, Deserialize)]
struct CollectionsParams {
    page: Option<u64>,
}

#[get("/nft_collections")]
async fn collections(info: Query<CollectionsParams>, state: WebData) -> impl Responder {
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
    let mut insc_list = db.get_inscriptions_by_id(&id_list);

    for insc in insc_list.iter_mut() {
        insc.mime_data = "".to_string();
    }

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
    let prefix = make_index_key(KEY_INSC_NFT_INDEX_HOLDER_ADDRESS, &info.address);
    let key_list = db.get_item_keys(&prefix, &prefix, page * PAGE_SIZE, PAGE_SIZE, Direction::Forward);
    let id_list = db_index2id_desc(key_list);
    let mut insc_list = db.get_inscriptions_by_id(&id_list);

    if !info.include_image_data.unwrap_or(false) {
        for insc in insc_list.iter_mut() {
            if insc.mime_category == InscriptionMimeCategory::Image {
                insc.mime_data = "".to_string();
            }
        }
    }

    HttpResponse::response_data(insc_list)
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
            InscriptionMimeCategory::Text => HttpResponse::Ok().body(insc.mime_data),
            _ => HttpResponse::BadRequest().body("Not an NFT"),
        },
        None => HttpResponse::NotFound().body("Inscription not found"),
    }
}
