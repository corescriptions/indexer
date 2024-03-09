use super::{
    db::{InscribeDB, InscribeTxn},
    types::{Indexer, WorkerInscribe, WorkerSync},
};
use crate::{
    config::{DB_PATH, REINDEX},
    inscription::db_checkpoint::rollback,
};
use log::info;
use rocksdb::TransactionDB;
use std::sync::{Arc, RwLock};

impl Indexer {
    pub fn new() -> Self {
        let txn_db = TransactionDB::open_default(DB_PATH.as_str()).unwrap();
        let db = Arc::new(RwLock::new(txn_db));

        let start_sync_blocknumber = db.read().unwrap().get_sync_blocknumber();
        info!("indexer start blocknumber: {}", start_sync_blocknumber);

        Indexer {
            db: db.clone(),
            worker_sync: Arc::new(WorkerSync::new(db.clone())),
            worker_inscribe: Arc::new(WorkerInscribe::new(db.clone())),
        }
    }

    fn delete_keys(&self, prefix: &str) {
        let db = self.db.write().unwrap();
        let mut total_delete_count = 0;
        loop {
            let txn = db.transaction();
            let delete_count = txn.delete_keys(prefix, 1000000);
            txn.commit().unwrap();
            total_delete_count += delete_count;
            info!("delete_keys total: {} {}", prefix, total_delete_count);
            if delete_count < 1000000 {
                break;
            }
        }
    }

    pub fn rollback(&mut self, blocknumber: u64) -> bool {
        if rollback(blocknumber) {
            self.db = Arc::new(RwLock::new(TransactionDB::open_default(DB_PATH.as_str()).unwrap()));
            info!("[checkpoint] rollback to: {}", blocknumber);
            true
        } else {
            false
        }
    }

    pub fn init(&mut self) {
        if *REINDEX {
            use super::db::*;
            use super::marketplace::db::*;
            // KEY_SYNC_BLOCKNUMBER
            // KEY_SYNC_BLOCK_HASH
            // KEY_INSC_SYNC_TOP
            self.delete_keys(KEY_INSC_TOP);

            // KEY_INSC_INDEX_ID
            // KEY_INSC_INDEX_TX
            self.delete_keys(KEY_INSC_INDEX_SIGN);
            self.delete_keys(KEY_INSC_INDEX_NFT_ID);
            // KEY_INSC_INDEX_CREATER
            self.delete_keys(KEY_INSC_INDEX_ADDRESS);

            self.delete_keys(KEY_INSC_NFT_INDEX_CREATER);
            self.delete_keys(KEY_INSC_NFT_INDEX_HOLDER);
            self.delete_keys(KEY_INSC_NFT_INDEX_HOLDER_ADDRESS);
            self.delete_keys(KEY_INSC_NFT_TRANS_INDEX_ID);
            self.delete_keys(KEY_INSC_NFT_COLL_INDEX_ID);

            self.delete_keys(KEY_INSC_TOKEN_INDEX_ID);
            self.delete_keys(KEY_INSC_TOKEN_INDEX_TICK);
            self.delete_keys(KEY_INSC_TOKEN_INDEX_TICK_I);
            self.delete_keys(KEY_INSC_BALANCE_INDEX_TICK_BALANCE_HOLDER);
            self.delete_keys(KEY_INSC_BALANCE_INDEX_HOLDER_TICK);

            self.delete_keys(KEY_MARKET_ORDER_INDEX_ID);
            self.delete_keys(KEY_MARKET_ORDER_INDEX_SELLER);
            self.delete_keys(KEY_MARKET_ORDER_INDEX_TICK_PRICE);
            self.delete_keys(KEY_MARKET_ORDER_INDEX_NFT);
            self.delete_keys(KEY_MARKET_ORDER_INDEX_TIME);
            self.delete_keys(KEY_MARKET_ORDER_INDEX_TICK_TIME);
            self.delete_keys(KEY_MARKET_ORDER_INDEX_SELLER_CLOSE_CANCEL);
            self.delete_keys(KEY_MARKET_ORDER_INDEX_CLOSE_TICK_TIME);

            info!("[indexer] reindex done");
        }
    }

    pub async fn run(&self) {
        WorkerSync::run(self.worker_sync.clone());
        self.worker_inscribe.run().await;
    }
}
