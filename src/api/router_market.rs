use super::{HttpResponseExt, WebData, PAGE_SIZE};
use crate::inscription::{
    db::*,
    marketplace::{db::*, types::*},
};
use actix_web::{get, web, web::Query, HttpResponse, Responder};
use rocksdb::Direction;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn register(config: &mut web::ServiceConfig) {
    config.service(market_orders_all);
    config.service(market_orders_list);
    config.service(market_orders);
    config.service(market_order);
}

async fn market_get_order_list(state: WebData, page: u64, prefix: &str) -> Vec<serde_json::value::Value> {
    let db = state.db.read().unwrap();
    let key_list = db.get_item_keys(&prefix, &prefix, page * PAGE_SIZE, PAGE_SIZE, Direction::Forward);
    let id_list = db_index2str(key_list);
    let mut order_list = Vec::new();
    for id in &id_list {
        let order = db.market_get_order_by_id(&id.to_string()).unwrap();
        order_list.push(market_order_to_display(&order));
    }
    order_list
}

fn str_option(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

fn order_status_str(status: &MarketOrderStatus) -> String {
    match status {
        MarketOrderStatus::Init => "init".to_string(),
        MarketOrderStatus::Open => "open".to_string(),
        MarketOrderStatus::Close => "close".to_string(),
        MarketOrderStatus::Canceled => "canceled".to_string(),
    }
}

pub fn market_order_to_display(order: &MarketOrder) -> serde_json::Value {
    match order.order_type {
        MarketOrderType::Token => json!({
            "order_type": "token",
            "order_id": order.order_id,
            "from": order.from,
            "to": order.to,
            "tick": order.tick,
            "nft_id": serde_json::Value::Null,
            "nft_tx": serde_json::Value::Null,
            "amount": order.amount.to_string(),
            "total_price": order.total_price.to_string(),
            "unit_price": order.unit_price.to_string(),
            "tx": order.tx,
            "tx_setprice": str_option(&order.tx_setprice),
            "tx_cancel": str_option(&order.tx_cancel),
            "tx_close": str_option(&order.tx_close),
            "blocknumber": order.blocknumber.to_string(),
            "timestamp": order.timestamp.to_string(),
            "order_status": order_status_str(&order.order_status),
            "buyer": str_option(&order.buyer),
        }),
        MarketOrderType::NFT => json!( {
            "order_type": "nft",
            "order_id": order.order_id,
            "from": order.from,
            "to": order.to,
            "tick": serde_json::Value::Null,
            "nft_id": order.nft_id.to_string(),
            "nft_tx": order.nft_tx,
            "amount": "1",
            "total_price": order.total_price.to_string(),
            "unit_price": order.total_price.to_string(),
            "tx": order.tx,
            "tx_setprice": str_option(&order.tx_setprice),
            "tx_cancel": str_option(&order.tx_cancel),
            "tx_close": str_option(&order.tx_close),
            "blocknumber": order.blocknumber.to_string(),
            "timestamp": order.timestamp.to_string(),
            "order_status": order_status_str(&order.order_status),
            "buyer": str_option(&order.buyer),
        }),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketOrderAllParams {
    page: Option<u64>,
    tick: Option<String>,
}

#[get("/market_orders_all")]
async fn market_orders_all(info: Query<MarketOrderAllParams>, state: WebData) -> impl Responder {
    let prefix = match &info.tick {
        Some(tick) => make_index_key(KEY_MARKET_ORDER_INDEX_TICK_TIME, tick) + ":",
        None => KEY_MARKET_ORDER_INDEX_TIME.to_string(),
    };
    let order_list = market_get_order_list(state, info.page.unwrap_or(1) - 1, &prefix).await;

    HttpResponse::response_data(order_list)
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketOrderListParams {
    page: Option<u64>,
    order_type: String,
    order_status: String,
    tick: Option<String>,
}

#[get("/market_orders_list")]
async fn market_orders_list(info: Query<MarketOrderListParams>, state: WebData) -> impl Responder {
    let prefix = if info.order_type == "token" {
        if let Some(tick) = &info.tick {
            if info.order_status == "open" {
                make_index_key(KEY_MARKET_ORDER_INDEX_TICK_PRICE, tick) + ":"
            } else if info.order_status == "close" {
                make_index_key(KEY_MARKET_ORDER_INDEX_CLOSE_TICK_TIME, tick) + ":"
            } else {
                return HttpResponse::response_error(1, "Invalid params");
            }
        } else {
            return HttpResponse::response_error(1, "Invalid params");
        }
    } else if info.order_type == "nft" {
        KEY_MARKET_ORDER_INDEX_NFT.to_string()
    } else {
        return HttpResponse::response_error(1, "Invalid params");
    };

    let order_list = market_get_order_list(state, info.page.unwrap_or(1) - 1, &prefix).await;

    HttpResponse::response_data(order_list)
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketMyOpenParams {
    page: Option<u64>,
    address: String,
    order_status: Option<String>,
}

#[get("/market_orders")]
async fn market_orders(info: Query<MarketMyOpenParams>, state: WebData) -> impl Responder {
    let prefix_index = match &info.order_status {
        Some(order_status) if order_status == "open" => KEY_MARKET_ORDER_INDEX_SELLER.to_string(),
        Some(order_status) if order_status == "close" => KEY_MARKET_ORDER_INDEX_SELLER_CLOSE_CANCEL.to_string(),
        None => KEY_MARKET_ORDER_INDEX_SELLER.to_string(),
        _ => return HttpResponse::response_error(1, "Invalid params"),
    };

    let prefix = make_index_key(&prefix_index, info.address.to_lowercase());
    let order_list = market_get_order_list(state, info.page.unwrap_or(1) - 1, &prefix).await;

    HttpResponse::response_data(order_list)
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketOrderParams {
    order_id: String,
}

#[get("/market_order")]
async fn market_order(info: Query<MarketOrderParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let order = db.market_get_order_by_id(&info.order_id);
    match order {
        Some(order) => HttpResponse::response_data(market_order_to_display(&order)),
        None => return HttpResponse::response_error_notfound(),
    }
}
