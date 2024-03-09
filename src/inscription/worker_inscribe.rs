use super::{
    db::InscribeDB,
    inscribe_patch::INSCRIBE_PATCH_DATA,
    types::{InscribeContext, InscribePatch, Inscription, WorkerInscribe},
};
use crate::global::sleep_ms;
use log::info;
use rocksdb::TransactionDB;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct InscribeFilterConfig {
    pub tx_filter: Vec<String>,
    pub block_filter: Vec<u64>,
    pub mint_pass_tx: Vec<String>,
}

impl WorkerInscribe {
    pub fn new(db: Arc<RwLock<TransactionDB>>) -> Self {
        WorkerInscribe {
            db,
            inscribe_patch: Self::load_inscribe_filter(),
        }
    }

    pub fn load_block(&self, blocknumber: u64) -> Vec<Inscription> {
        let mut insc_list = Vec::new();
        let db = self.db.read().unwrap();
        let mut insc_id = db.get_top_inscription_id() + 1;
        loop {
            if let Some(insc) = db.get_inscription_by_id(insc_id) {
                if insc.blocknumber != blocknumber {
                    break;
                }
                insc_list.push(insc);
                insc_id += 1;
            } else {
                break;
            }
        }
        insc_list
    }

    pub async fn inscribe(&self) -> bool {
        let insc_id = self.db.read().unwrap().get_top_inscription_id();
        let sync_id = self.db.read().unwrap().get_top_inscription_sync_id();
        let sync_blocknumber = self.db.read().unwrap().get_sync_blocknumber();

        if insc_id >= sync_id {
            info!("[indexer] inscribe: wait for new inscription");
            return false;
        }

        let insc = self.db.read().unwrap().get_inscription_by_id(insc_id + 1).unwrap();
        let current_blocknumber = insc.blocknumber;
        if current_blocknumber >= sync_blocknumber {
            info!("[indexer] inscribe: wait for new block");
            return false;
        }

        let insc_list = self.load_block(current_blocknumber);

        let mut context = InscribeContext::new(self.db.clone(), &self.inscribe_patch);
        context.inscriptions = insc_list;
        context.inscribe();
        context.save();

        return true;
    }

    fn load_inscribe_filter() -> InscribePatch {
        let filter_config: InscribeFilterConfig = match serde_json::from_str(&INSCRIBE_PATCH_DATA) {
            Ok(filter) => filter,
            Err(_) => {
                panic!("Unable to parse inscribe_filter.json");
            }
        };

        let mut tx_filter_set = HashSet::new();
        for tx in &filter_config.tx_filter {
            tx_filter_set.insert(tx.to_string());
        }

        let mut block_filter_set = HashSet::new();
        for block in &filter_config.block_filter {
            block_filter_set.insert(*block);
        }

        let mut mint_pass_tx_set = HashSet::new();
        for tx in &filter_config.mint_pass_tx {
            mint_pass_tx_set.insert(tx.to_string());
        }

        InscribePatch {
            tx_filter: tx_filter_set,
            block_filter: block_filter_set,
            mint_pass_tx: mint_pass_tx_set,
        }
    }

    async fn run_inscribe(&self) {
        loop {
            if !self.inscribe().await {
                sleep_ms(3000).await;
            }
        }
    }

    pub fn run(arc_self: Arc<Self>) {
        let arc_self1 = arc_self.clone();
        tokio::spawn(async move {
            arc_self1.run_inscribe().await;
        });
    }
}
