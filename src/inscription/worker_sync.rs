use super::{
    db::{InscribeDB, InscribeTxn},
    db_checkpoint::{checkpoints_list, make_checkpoint},
    trait_tx::TrailsTx,
    types::{WorkerSync, WorkerSyncState},
};
use crate::{
    config::{
        CHECKPOINT_SPAN, CONFIRM_BLOCK, FINALIZED_BLOCK, MARKET_ADDRESS_LIST, START_BLOCK, WEB3_PROVIDER, WORKER_BUFFER_LENGTH,
        WORKER_COUNT,
    },
    ethereum::{init_web3_http, HexParseTrait, Web3Ex},
    global::{get_timestamp_ms, sleep_ms, ROLLBACK_BLOCK},
};
use log::{error, info, warn};
use rocksdb::TransactionDB;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

impl WorkerSync {
    pub fn new(db: Arc<RwLock<rocksdb::TransactionDB>>) -> Self {
        WorkerSync {
            db,
            state: Arc::new(RwLock::new(WorkerSyncState {
                blocks: HashMap::new(),
                event_logs: HashMap::new(),
                worker_count: 0,
                latest_blocknumber: 0,
            })),
        }
    }

    async fn check_finalized_block_hash(db: Arc<RwLock<TransactionDB>>, blocknumber: u64) {
        let web3 = init_web3_http(WEB3_PROVIDER.to_string().as_str());
        let block_hash_latest = db.read().unwrap().get_block_hash(blocknumber);
        if block_hash_latest.is_none() {
            return;
        }

        let block_hash_latest = block_hash_latest.unwrap();
        let finalized_block = web3.get_block_info_wait(blocknumber).await;
        let block_hash_finanlized = finalized_block.hash.unwrap().to_hex_string();

        if block_hash_latest != block_hash_finanlized {
            error!(
                "[sync] sync error block: {}, finalized hash: {}, latest hash: {}",
                blocknumber, block_hash_finanlized, block_hash_latest
            );
            
            let consensus_block = Self::find_consensus_checkpoint(db, blocknumber).await;
            if consensus_block > 0 {
                *ROLLBACK_BLOCK.lock().unwrap() = consensus_block;
                info!("[sync] rollback to block: {}", consensus_block);
            } else {
                panic!("cannot find consensus block");
            }
        }
    }

    async fn find_consensus_block(db: Arc<RwLock<TransactionDB>>, start_blocknumber: u64) -> u64 {
        let web3 = init_web3_http(WEB3_PROVIDER.to_string().as_str());
        let mut blocknumber = start_blocknumber - 1;
        loop {
            let block_hash_db = db.read().unwrap().get_block_hash(blocknumber).unwrap();
            let block = web3.get_block_wait(blocknumber).await;
            let block_hash_now = block.hash.unwrap().to_hex_string();
            if block_hash_db == block_hash_now {
                info!("[sync] find consensus block: {}", blocknumber);
                return blocknumber;
            }
            warn!(
                "[sync] consensus error: {}, db: {}, now: {}",
                blocknumber, block_hash_db, block_hash_now
            );
            blocknumber -= 1;
        }
    }

    async fn find_consensus_checkpoint(db: Arc<RwLock<TransactionDB>>, start_blocknumber: u64) -> u64 {
        let consensus_block = Self::find_consensus_block(db, start_blocknumber).await;
        let checklists = checkpoints_list();
        for checkpoint in checklists.iter().rev() {
            if *checkpoint <= consensus_block {
                return *checkpoint;
            }
        }

        0
    }

    fn co_fecth_block(&self, blocknumber: u64) {
        let state = self.state.clone();
        let db = self.db.clone();

        state.write().unwrap().worker_count += 1;

        tokio::spawn(async move {
            let web3 = init_web3_http(WEB3_PROVIDER.to_string().as_str());
            let block = web3.get_block_wait(blocknumber).await;
            let tx_count = block.transactions.len();

            let mut market_event_enalbe = false;
            for tx in &block.transactions {
                if tx.to.is_some() && MARKET_ADDRESS_LIST.contains(&tx.to.unwrap().to_hex_string().to_lowercase()) {
                    market_event_enalbe = true;
                    break;
                }
            }

            let block_logs = if market_event_enalbe {
                Some(web3.get_event_logs_wait(&MARKET_ADDRESS_LIST, blocknumber).await)
            } else {
                None
            };

            if state.read().unwrap().latest_blocknumber - blocknumber < *FINALIZED_BLOCK {
                Self::check_finalized_block_hash(db, blocknumber).await;
            }

            let mut state = state.write().unwrap();
            state.blocks.insert(blocknumber, block);

            if let Some(block_logs) = block_logs {
                if block_logs.len() > 0 {
                    state.event_logs.insert(blocknumber, block_logs);
                }
            }
            state.worker_count -= 1;
            info!(
                "[sync] fetch block: {}, txs: {}, workers: {}, buff_len: {}",
                blocknumber,
                tx_count,
                state.worker_count,
                state.blocks.len()
            );
        });
    }

    pub fn save_block(&self, blocknumber: u64) {
        let mut sync_state = self.state.write().unwrap();
        let block = sync_state.blocks.remove(&blocknumber).unwrap();
        let block_logs = sync_state.event_logs.remove(&blocknumber);
        let db = self.db.write().unwrap();

        let mut next_insc_id = db.get_top_inscription_sync_id() + 1;
        let mut inscription_count = 0;

        let txn = db.transaction();
        for tx in &block.transactions {
            let tx_index = tx.transaction_index.unwrap().as_u64();
            let tx_logs = match &block_logs {
                Some(_tx_logs) => _tx_logs.get(&tx_index),
                None => None,
            };
            if let Some(insc) = tx.to_inscription(&block, tx_logs, next_insc_id) {
                txn.inscription_insert(&insc);
                next_insc_id += 1;
                inscription_count += 1;
            }
        }

        txn.set_block_hash(blocknumber, &block.hash.unwrap().to_hex_string());
        txn.set_sync_blocknumber(blocknumber);
        txn.set_top_inscription_sync_id(next_insc_id - 1);
        txn.commit().unwrap();

        info!(
            "[sync] save block: {}, txs: {}, ins: {}",
            blocknumber,
            block.transactions.len(),
            inscription_count
        );
    }

    pub async fn run_save(&self) {
        let start_blocknumber = self.get_sync_blocknumber();
        let start_time = get_timestamp_ms();

        loop {
            let blocknumber = self.get_sync_blocknumber();
            let saved;
            let block_existed = self.state.read().unwrap().blocks.contains_key(&blocknumber);
            if block_existed {
                self.save_block(blocknumber);
                saved = true;

                let now = get_timestamp_ms();
                info!(
                    "[sync] sync block speed: {} blk/s",
                    ((blocknumber - start_blocknumber) as f64 / (now - start_time) as f64 * 100_000f64) as u64 / 100
                );
            } else {
                saved = false;
            }

            if saved {
                if blocknumber % *CHECKPOINT_SPAN == 0 {
                    make_checkpoint(blocknumber);
                }
            } else {
                sleep_ms(10).await;
                continue;
            }
        }
    }

    fn get_sync_blocknumber(&self) -> u64 {
        let sync_blocknumber = self.db.read().unwrap().get_sync_blocknumber();
        if sync_blocknumber < *START_BLOCK {
            *START_BLOCK
        } else {
            sync_blocknumber + 1
        }
    }

    pub async fn run_sync(&self) {
        let web3 = init_web3_http(WEB3_PROVIDER.to_string().as_str());
        let start_blocknumber = self.get_sync_blocknumber();

        info!(
            "[sync] start sync blocknumber: {}, latest: {}",
            start_blocknumber,
            web3.get_blocknumber_wait().await
        );

        let mut next_blocknumber = start_blocknumber;
        loop {
            if *WORKER_COUNT == self.state.read().unwrap().worker_count {
                sleep_ms(10).await;
                continue;
            }

            let latest_blocknumber = web3.get_blocknumber_wait().await;
            let target_blocknumber = latest_blocknumber - *CONFIRM_BLOCK;

            self.state.write().unwrap().latest_blocknumber = latest_blocknumber;

            if next_blocknumber > target_blocknumber {
                info!("[sync] wait for new block: {}", next_blocknumber);
                sleep_ms(3000).await;
                continue;
            }

            if self.state.read().unwrap().worker_count >= *WORKER_COUNT
                || self.state.read().unwrap().blocks.len() > *WORKER_BUFFER_LENGTH
            {
                sleep_ms(10).await;
                continue;
            }

            let launch_worker_count = std::cmp::min(
                target_blocknumber - next_blocknumber + 1,
                *WORKER_COUNT - self.state.read().unwrap().worker_count,
            );

            if launch_worker_count == 0 {
                sleep_ms(10).await;
                continue;
            }

            for _ in 0..launch_worker_count {
                self.co_fecth_block(next_blocknumber);
                info!(
                    "[sync] new block: {}, latest: {}, workers: {}",
                    next_blocknumber,
                    latest_blocknumber,
                    self.state.read().unwrap().worker_count,
                );
                next_blocknumber += 1;
            }
        }
    }

    pub fn run(arc_self: Arc<Self>) {
        let arc_self1 = arc_self.clone();
        tokio::spawn(async move {
            arc_self1.run_sync().await;
        });

        let arc_self2 = arc_self.clone();
        tokio::spawn(async move {
            arc_self2.run_save().await;
        });
    }
}
