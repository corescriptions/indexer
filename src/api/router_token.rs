use super::{HttpResponseExt, WebData, PAGE_SIZE};
use crate::inscription::{db::*, types::*};
use actix_web::{get, web, web::Query, HttpResponse, Responder};
use regex::Regex;
use serde::{Deserialize, Serialize};

pub fn register(config: &mut web::ServiceConfig) {
    config.service(tokens);
    config.service(token_holders);
    config.service(token_balance);
}

fn contains_chinese_characters(s: &str) -> bool {
    let pattern = Regex::new(r"[\u{4e00}-\u{9fa5}]").unwrap();
    pattern.is_match(s)
}

#[derive(Debug, Serialize, Deserialize)]
struct TokensParams {
    page: Option<u64>,
    order_by: Option<String>,
    mint_finished: Option<bool>,
}

#[get("/tokens")]
async fn tokens(info: Query<TokensParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let mut list = db.get_tokens_list();
    let order_type = &info.order_by;

    match order_type.as_deref() {
        Some("holders") => {
            list.sort_unstable_by(|a: &InscriptionToken, b: &InscriptionToken| b.holders.cmp(&a.holders));
        }
        _ => {}
    }
    if info.mint_finished.unwrap_or(false) {
        list.retain(|i| i.mint_finished);
    }

    let mut first_list = Vec::new();
    let mut second_list = Vec::new();
    for i in list {
        if i.tick.len() <= 8 && !contains_chinese_characters(&i.tick) {
            if i.tick == "coco" {
                first_list.push(i);
            } else {
                second_list.push(i);
            }
        }
    }
    let result = first_list.into_iter().chain(second_list.into_iter()).collect::<Vec<_>>();
    let mut pages = result.chunks(PAGE_SIZE.try_into().unwrap());
    if let Some(page_slice) = pages.nth(page.try_into().unwrap()) {
        HttpResponse::response_data(page_slice)
    } else {
        HttpResponse::response_data(Vec::<InscriptionToken>::new())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenHoldersParams {
    page: Option<u64>,
    tick: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenHoldersResponse {
    address: String,
    balance: u64,
}

#[get("/token_holders")]
async fn token_holders(info: Query<TokenHoldersParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let start_key = make_index_key(KEY_INSC_BALANCE_INDEX_TICK_BALANCE_HOLDER, &info.tick);
    let key_list = db.get_items(
        &start_key,
        &start_key,
        page * PAGE_SIZE,
        PAGE_SIZE,
        rocksdb::Direction::Forward,
    );

    let mut holders: Vec<TokenHoldersResponse> = Vec::new();
    for (key, value) in &key_list {
        let address = String::from_utf8(key[key.len() - 42..].to_vec()).unwrap();
        let balance = u64::from_be_bytes(value.as_slice().try_into().unwrap());
        holders.push(TokenHoldersResponse { address, balance });
    }

    HttpResponse::response_data(holders)
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenBalanceParams {
    address: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenBalanceResponse {
    tick: String,
    balance: u64,
}

#[get("/token_balance")]
async fn token_balance(info: Query<TokenBalanceParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let prefix = make_index_key(KEY_INSC_BALANCE_INDEX_HOLDER_TICK, info.address.to_lowercase().as_str());
    let key_list = db.get_items(&prefix, &prefix, 0, 9999, rocksdb::Direction::Forward);
    let mut token_list = Vec::new();
    for (key, value) in &key_list {
        let tick_pos = key.len() - key.iter().rev().position(|&x| x == b':').unwrap();
        let tick = String::from_utf8(key[tick_pos..].to_vec()).unwrap();
        let balance = u64::from_be_bytes(value.as_slice().try_into().unwrap());
        token_list.push(TokenBalanceResponse { tick, balance });
    }
    HttpResponse::response_data(token_list)
}
