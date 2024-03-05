use super::types::{MarketOrder, MarketOrderStatus, MarketOrderType};
use crate::{
    inscription::db::{make_index_key, make_index_key2, make_index_key3},
    num_index, num_index_desc,
    txn_db::{DBBase, TxnDB},
};
use rocksdb::{DBAccess, Transaction, TransactionDB};

pub const KEY_MARKET_ORDER_INDEX_ID: &'static str = "market_id";
pub const KEY_MARKET_ORDER_INDEX_SELLER: &'static str = "market_seller-sort-id";
pub const KEY_MARKET_ORDER_INDEX_TICK_PRICE: &'static str = "market_tick_price-id";
pub const KEY_MARKET_ORDER_INDEX_NFT: &'static str = "market_nft_id";
pub const KEY_MARKET_ORDER_INDEX_TIME: &'static str = "market_time-id";
pub const KEY_MARKET_ORDER_INDEX_TICK_TIME: &'static str = "market_tick_time-id";
pub const KEY_MARKET_ORDER_INDEX_SELLER_CLOSE_CANCEL: &'static str = "market_seller_close_cancel-sort-id";

pub trait InscribeMarketDB: TxnDB {
    fn market_get_order_by_id(&self, order_id: &str) -> Option<MarketOrder>;
}

impl<T: DBBase + TxnDB + DBAccess> InscribeMarketDB for T {
    fn market_get_order_by_id(&self, order_id: &str) -> Option<MarketOrder> {
        let index_key_id = make_index_key(KEY_MARKET_ORDER_INDEX_ID, &order_id);
        match self.get(index_key_id.as_bytes()).unwrap() {
            Some(data) => Some(serde_json::from_slice(&data).unwrap()),
            None => None,
        }
    }
}

pub trait InscribeMarketTxn<'a> {
    fn market_order_save(&self, order: &MarketOrder);
    fn market_order_set_price(&self, db: &TransactionDB, tx_hash: &str, order_id: &str, total_price: u128);
    fn market_order_cancel(&self, db: &TransactionDB, tx_hash: &str, order_id: &str);
    fn market_order_close(&self, db: &TransactionDB, tx_hash: &str, order_id: &str, buyer: &str);
}

impl<'a> InscribeMarketTxn<'a> for Transaction<'a, TransactionDB> {
    fn market_order_save(&self, order: &MarketOrder) {
        let order_data = serde_json::to_string(order).unwrap();

        let index_key_id = make_index_key(KEY_MARKET_ORDER_INDEX_ID, &order.order_id);
        let index_key_seller = make_index_key3(
            KEY_MARKET_ORDER_INDEX_SELLER,
            &order.from,
            num_index_desc!(order.blocknumber),
            &order.order_id,
        );
        let index_key_time = make_index_key2(KEY_MARKET_ORDER_INDEX_TIME, num_index_desc!(order.timestamp), &order.order_id);
        let index_key_tick_time = make_index_key3(
            KEY_MARKET_ORDER_INDEX_TICK_TIME,
            &order.tick,
            num_index_desc!(order.timestamp),
            &order.order_id,
        );

        self.put(index_key_id.as_bytes(), order_data.as_bytes()).unwrap();
        self.put(index_key_seller.as_bytes(), "").unwrap();
        self.put(index_key_time.as_bytes(), "").unwrap();
        self.put(index_key_tick_time.as_bytes(), "").unwrap();
    }

    fn market_order_set_price(&self, db: &TransactionDB, tx_hash: &str, order_id: &str, total_price: u128) {
        let mut order = db.market_get_order_by_id(&order_id).unwrap();
        order.total_price = total_price;
        order.unit_price = total_price / order.amount as u128;
        order.tx_setprice = tx_hash.to_string();
        order.order_status = MarketOrderStatus::Open;

        let order_data = serde_json::to_string(&order).unwrap();
        let index_key_id = make_index_key(KEY_MARKET_ORDER_INDEX_ID, &order.order_id);
        self.put(index_key_id.as_bytes(), order_data.as_bytes()).unwrap();

        match order.order_type {
            MarketOrderType::NFT => {
                let index_key_nft = make_index_key2(KEY_MARKET_ORDER_INDEX_NFT, num_index_desc!(order.timestamp), order.nft_id);
                self.put(index_key_nft.as_bytes(), "").unwrap();
            }
            MarketOrderType::Token => {
                let index_key_tick_price = make_index_key3(
                    KEY_MARKET_ORDER_INDEX_TICK_PRICE,
                    &order.tick,
                    num_index!(order.unit_price),
                    &order.order_id,
                );

                self.put(index_key_tick_price.as_bytes(), "").unwrap();
            }
        }
    }

    fn market_order_cancel(&self, db: &TransactionDB, tx_hash: &str, order_id: &str) {
        let mut order = db.market_get_order_by_id(&order_id).unwrap();
        order.tx_cancel = tx_hash.to_string();
        order.order_status = MarketOrderStatus::Canceled;

        let order_data = serde_json::to_string(&order).unwrap();
        let index_key_id = make_index_key(KEY_MARKET_ORDER_INDEX_ID, &order.order_id);
        self.put(index_key_id.as_bytes(), order_data.as_bytes()).unwrap();

        let index_key_seller = make_index_key3(
            KEY_MARKET_ORDER_INDEX_SELLER,
            &order.from,
            num_index_desc!(order.blocknumber),
            &order.order_id,
        );
        self.delete(index_key_seller.as_bytes()).unwrap();

        let index_key_seller_close_cancel = make_index_key3(
            KEY_MARKET_ORDER_INDEX_SELLER_CLOSE_CANCEL,
            &order.from,
            num_index_desc!(order.blocknumber),
            &order.order_id,
        );
        self.put(index_key_seller_close_cancel.as_bytes(), "").unwrap();

        if order.order_status == MarketOrderStatus::Open {
            match order.order_type {
                MarketOrderType::NFT => {
                    let index_key_nft =
                        make_index_key2(KEY_MARKET_ORDER_INDEX_NFT, num_index_desc!(order.timestamp), order.nft_id);
                    self.delete(index_key_nft.as_bytes()).unwrap();
                }
                MarketOrderType::Token => {
                    let index_key_tick_price = make_index_key3(
                        KEY_MARKET_ORDER_INDEX_TICK_PRICE,
                        &order.tick,
                        num_index!(order.unit_price),
                        &order.order_id,
                    );

                    self.delete(index_key_tick_price.as_bytes()).unwrap();
                }
            }
        }
    }

    fn market_order_close(&self, db: &TransactionDB, tx_hash: &str, order_id: &str, buyer: &str) {
        let mut order = db.market_get_order_by_id(&order_id).unwrap();
        order.tx_close = tx_hash.to_string();
        order.buyer = buyer.to_string();
        order.order_status = MarketOrderStatus::Close;

        let order_data = serde_json::to_string(&order).unwrap();
        let index_key_id = make_index_key(KEY_MARKET_ORDER_INDEX_ID, &order.order_id);
        self.put(index_key_id.as_bytes(), order_data.as_bytes()).unwrap();

        let index_key_seller = make_index_key3(
            KEY_MARKET_ORDER_INDEX_SELLER,
            &order.from,
            num_index_desc!(order.blocknumber),
            &order.order_id,
        );
        self.delete(index_key_seller.as_bytes()).unwrap();

        let index_key_seller_close_cancel = make_index_key3(
            KEY_MARKET_ORDER_INDEX_SELLER_CLOSE_CANCEL,
            &order.from,
            num_index_desc!(order.blocknumber),
            &order.order_id,
        );
        self.put(index_key_seller_close_cancel.as_bytes(), "").unwrap();

        match order.order_type {
            MarketOrderType::NFT => {
                let index_key_nft = make_index_key2(KEY_MARKET_ORDER_INDEX_NFT, num_index_desc!(order.timestamp), order.nft_id);
                self.delete(index_key_nft.as_bytes()).unwrap();
            }
            MarketOrderType::Token => {
                let index_key_tick_price = make_index_key3(
                    KEY_MARKET_ORDER_INDEX_TICK_PRICE,
                    &order.tick,
                    num_index!(order.unit_price),
                    &order.order_id,
                );

                self.delete(index_key_tick_price.as_bytes()).unwrap();
            }
        }
    }
}
