use super::{HttpResponseExt, WebData, PAGE_SIZE};
use crate::inscription::{
    db::*,
    marketplace::{db::*, types::*},
};
use actix_web::{get, web, web::Query, HttpResponse, Responder};
use rocksdb::Direction;
use serde::{Deserialize, Serialize};

pub fn register(config: &mut web::ServiceConfig) {
    config.service(market_orders_all);
    config.service(market_orders);
    config.service(market_order);
}

async fn market_get_order_list(state: WebData, page: u64, prefix: &str) -> Vec<MarketOrderDisplay> {
    let db = state.db.read().unwrap();
    let key_list = db.get_item_keys(&prefix, &prefix, page * PAGE_SIZE, PAGE_SIZE, Direction::Forward);
    let id_list = db_index2str(key_list);
    let mut order_list = Vec::new();
    for id in &id_list {
        let order = db.market_get_order_by_id(&id.to_string()).unwrap();
        order_list.push(order.to_display());
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrderDisplay {
    pub order_type: String,
    pub order_id: String,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nft_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nft_tx: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    pub total_price: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_price: Option<String>,
    pub tx: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_setprice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_cancel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_close: Option<String>,

    pub blocknumber: u64,
    pub timestamp: u64,

    pub order_status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer: Option<String>,
}

pub trait MarketOrderTrait {
    fn to_display(&self) -> MarketOrderDisplay;
}

impl MarketOrderTrait for MarketOrder {
    fn to_display(&self) -> MarketOrderDisplay {
        match self.order_type {
            MarketOrderType::Token => MarketOrderDisplay {
                order_type: "token".to_string(),
                order_id: self.order_id.to_string(),
                from: self.from.to_string(),
                to: self.to.to_string(),
                tick: Some(self.tick.to_string()),
                nft_id: None,
                nft_tx: None,
                amount: Some(self.amount.to_string()),
                total_price: self.total_price.to_string(),
                unit_price: Some(self.unit_price.to_string()),
                tx: self.tx.to_string(),
                tx_setprice: str_option(&self.tx_setprice),
                tx_cancel: str_option(&self.tx_cancel),
                tx_close: str_option(&self.tx_close),
                blocknumber: self.blocknumber,
                timestamp: self.timestamp,
                order_status: order_status_str(&self.order_status),
                buyer: str_option(&self.buyer),
            },
            MarketOrderType::NFT => MarketOrderDisplay {
                order_type: "nft".to_string(),
                order_id: self.order_id.to_string(),
                from: self.from.to_string(),
                to: self.to.to_string(),
                tick: None,
                nft_id: Some(self.nft_id),
                nft_tx: Some(self.nft_tx.to_string()),
                amount: None,
                total_price: self.total_price.to_string(),
                unit_price: None,
                tx: self.tx.to_string(),
                tx_setprice: str_option(&self.tx_setprice),
                tx_cancel: str_option(&self.tx_cancel),
                tx_close: str_option(&self.tx_close),
                blocknumber: self.blocknumber,
                timestamp: self.timestamp,
                order_status: order_status_str(&self.order_status),
                buyer: str_option(&self.buyer),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketOrderAllParams {
    page: Option<u64>,
    tick: Option<String>,
}

#[get("market_orders_all")]
async fn market_orders_all(info: Query<MarketOrderAllParams>, state: WebData) -> impl Responder {
    let prefix = match &info.tick {
        Some(tick) => make_index_key(KEY_MARKET_ORDER_INDEX_TICK_TIME, tick),
        None => KEY_MARKET_ORDER_INDEX_TIME.to_string(),
    };
    let order_list = market_get_order_list(state, info.page.unwrap_or(1) - 1, &prefix).await;

    HttpResponse::response_data(order_list)
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketOrderListParams {
    page: Option<u64>,
    order_type: Option<String>,
    tick: Option<String>,
}

#[get("market_orders_list")]
async fn market_orders_list(info: Query<MarketOrderListParams>, state: WebData) -> impl Responder {
    let prefix = match &info.order_type {
        Some(order_type) if order_type == "token" => {
            if let Some(tick) = &info.tick {
                make_index_key(KEY_MARKET_ORDER_INDEX_TICK_PRICE, tick)
            } else {
                return HttpResponse::response_error(1, "Invalid params");
            }
        }
        Some(order_type) if order_type == "nft" => KEY_MARKET_ORDER_INDEX_NFT.to_string(),
        None => KEY_MARKET_ORDER_INDEX_TIME.to_string(),
        _ => return HttpResponse::response_error(1, "Invalid params"),
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

#[get("market_orders")]
async fn market_orders(info: Query<MarketMyOpenParams>, state: WebData) -> impl Responder {
    let prefix_index = match &info.order_status {
        Some(order_status) if order_status == "open" => KEY_MARKET_ORDER_INDEX_SELLER.to_string(),
        Some(order_status) if order_status == "close" => KEY_MARKET_ORDER_INDEX_SELLER_CLOSE_CANCEL.to_string(),
        None => KEY_MARKET_ORDER_INDEX_SELLER.to_string(),
        _ => return HttpResponse::response_error(1, "Invalid params"),
    };

    let prefix = make_index_key(&prefix_index, info.address.to_ascii_lowercase());
    let order_list = market_get_order_list(state, info.page.unwrap_or(1) - 1, &prefix).await;

    HttpResponse::response_data(order_list)
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketOrderParams {
    order_id: String,
}

#[get("market_order")]
async fn market_order(info: Query<MarketOrderParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let order = db.market_get_order_by_id(&info.order_id);
    match order {
        Some(order) => HttpResponse::response_data(order.to_display()),
        None => return HttpResponse::response_error_notfound(),
    }
}
