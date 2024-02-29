use super::{
    db::{InscribeDB, InscribeTxn},
    types::{InscribeContext, Inscription},
};
use log::{info, warn};
use rocksdb::{Transaction, TransactionDB};
pub const APP_OPER_COLLECTION_DEPLOY: &'static str = "deoply";

pub trait ProcessBlockContextJsonCollection {
    fn execute_app_collection(&self, insc: &Inscription) -> bool;
    fn save_inscribe_collection(&self, txn: &rocksdb::Transaction<rocksdb::TransactionDB>, insc: &Inscription);
}

impl<'a> ProcessBlockContextJsonCollection for InscribeContext<'a> {
    fn execute_app_collection(&self, insc: &Inscription) -> bool {
        let oper = insc.json["op"].as_str();
        if oper.is_none() {
            warn!("[indexer] inscribe collection null oper: {}", insc.tx_hash);
            return false;
        }

        let oper = oper.unwrap();
        if oper != APP_OPER_COLLECTION_DEPLOY {
            warn!("[indexer] inscribe collection invalid oper {}: {}", insc.tx_hash, oper);
            return false;
        }

        let json = &insc.json;
        if !json["items"].is_array()
            || !json["name"].is_string()
            || !json["description"].is_string()
            || !json["url"].is_string()
            || !json["image"].is_string()
            || !json["icon"].is_string()
        {
            return false;
        }

        let items = json["items"].as_array().unwrap();
        let mut collection_items = Vec::<String>::new();
        for item in items {
            let item_tx_hash = item.as_str();
            if item_tx_hash.is_none() {
                return false;
            }

            let item_tx_hash = item_tx_hash.unwrap();
            let item_insc = self.db.read().unwrap().get_inscription_by_tx(item_tx_hash);
            if item_insc.is_none() {
                return false;
            }

            let item_holder = self
                .db
                .read()
                .unwrap()
                .get_inscription_nft_holder_by_id(item_insc.unwrap().id)
                .unwrap();

            if item_holder != insc.from {
                return false;
            }
            collection_items.push(item_tx_hash.to_string());
        }

        info!(
            "[indexer] inscribe collection {}: {}",
            insc.tx_hash.as_str(),
            json["name"].as_str().unwrap()
        );

        true
    }

    fn save_inscribe_collection(&self, txn: &Transaction<TransactionDB>, insc: &Inscription) {
        txn.inscription_nft_collection_insert(insc);
    }
}
