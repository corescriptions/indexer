use super::{
    db::{InscribeDB, InscribeTxn},
    inscribe_json::ProcessBlockContextJson,
    marketplace::MarketPlace,
    types::*,
};
use crate::config::MARKET_ADDRESS_LIST;
use log::info;
use rocksdb::{Transaction, TransactionDB};
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

impl<'a> InscribeContext<'a> {
    pub fn new(db: Arc<RwLock<rocksdb::TransactionDB>>, inscribe_filter: &'a InscribePatch) -> Self {
        InscribeContext {
            db: db.clone(),
            inscriptions: Vec::new(),
            nft_holders: HashMap::new(),
            nft_transfers: Vec::new(),
            nft_signatures: HashMap::new(),
            token_cache: db.read().unwrap().get_tokens(),
            token_balance_change: HashMap::new(),
            token_transfers: Vec::new(),
            inscribe_patch: inscribe_filter,
        }
    }

    pub fn save(&mut self) {
        let db = self.db.clone();
        let db = db.write().unwrap();
        let txn = db.transaction();

        for insc in &self.inscriptions {
            txn.inscription_inscribe(insc);
            if insc.verified == InscriptionVerifiedStatus::Successful {
                if insc.mime_category == InscriptionMimeCategory::Json {
                    self.save_inscribe_json(&db, &txn, &insc);
                }
                self.save_market(&db, &txn, &insc);
            }
        }

        txn.set_top_inscription_id(self.inscriptions.last().unwrap().id);

        self.save_token(&db, &txn);
        self.save_token_transfer(&txn);
        self.save_nft_transfer(&db, &txn);

        txn.commit().unwrap();

        info!(
            "[indexer] inscribe inscriptions: {} + {}, blocknumber: {} -> {}",
            self.inscriptions.first().unwrap().id,
            self.inscriptions.len(),
            self.inscriptions.first().unwrap().blocknumber,
            self.inscriptions.last().unwrap().blocknumber
        );
    }

    fn save_token(&mut self, db: &TransactionDB, txn: &Transaction<TransactionDB>) {
        for (_, token) in &self.token_cache {
            if token.deploy {
                txn.inscription_token_insert(token);
            }
        }

        for (tick, balance_change_coll) in &self.token_balance_change {
            let token = self.token_cache.get_mut(tick).unwrap();
            token.updated = true;
            for (address, balance_change) in balance_change_coll {
                let holder_change = txn.inscription_token_banalce_update(db, tick, address, *balance_change);
                token.holders = (token.holders as i64 + holder_change) as u64;
                if MARKET_ADDRESS_LIST.contains(address) {
                    token.market_updated = true;
                }
            }
        }

        for (_, token) in &mut self.token_cache {
            if token.market_updated {
                Self::update_token_market_info(db, token);
            }
            if token.updated {
                txn.inscription_token_update(token);
            }
        }
    }

    pub fn inscribe(&mut self) {
        let mut inscriptions = std::mem::take(&mut self.inscriptions);

        for insc in &mut inscriptions {
            if self.inscribe_patch.tx_filter.contains(&insc.tx_hash)
                || self.inscribe_patch.block_filter.contains(&insc.blocknumber)
            {
                insc.verified = InscriptionVerifiedStatus::Failed;
            } else {
                self.process_inscribe(insc);
            }
        }

        self.inscriptions = inscriptions;
    }

    fn process_inscribe(&mut self, insc: &mut Inscription) {
        let ins_result = match insc.mime_category {
            InscriptionMimeCategory::Transfer => self.process_inscribe_nft_transfer(insc),
            InscriptionMimeCategory::Json => self.process_inscribe_json(insc),
            InscriptionMimeCategory::Text | InscriptionMimeCategory::Image => self.process_inscribe_plain(insc),
            InscriptionMimeCategory::Invoke => self.process_inscribe_invoke(insc),
            _ => false,
        };

        insc.verified = if ins_result {
            InscriptionVerifiedStatus::Successful
        } else {
            InscriptionVerifiedStatus::Failed
        }
    }

    fn process_inscribe_plain(&mut self, insc: &mut Inscription) -> bool {
        let mut hasher = Sha1::new();
        hasher.update(insc.mime_data.as_bytes());
        let result = hasher.finalize();
        let signature = format!("{:x}", result);
        if self.nft_signature_exists(signature.as_str()) {
            info!("[indexer] inscribe existed: {} {}", insc.tx_hash.as_str(), signature);
            return false;
        }

        self.nft_signatures.insert(signature.clone(), insc.id);
        self.nft_holders.insert(insc.id, insc.to.clone());

        insc.signature = Some(signature);

        info!("[indexer] inscribe {}: {}", insc.mime_type, insc.tx_hash.as_str());
        true
    }

    pub fn get_nft_holder(&self, insc_id: u64) -> String {
        if let Some(holder) = self.nft_holders.get(&insc_id) {
            holder.to_string()
        } else {
            let holder = self.db.read().unwrap().get_inscription_nft_holder_by_id(insc_id).unwrap();
            holder
        }
    }

    pub fn nft_signature_exists(&self, signature: &str) -> bool {
        match self.nft_signatures.get(signature) {
            Some(_) => true,
            None => self.db.read().unwrap().inscription_sign_exists(signature),
        }
    }

    fn process_inscribe_nft_transfer(&mut self, insc: &Inscription) -> bool {
        let mut trans = Vec::new();

        for i in (0..insc.mime_data.len()).step_by(TRANSFER_TX_HEX_LENGTH) {
            let item_insc_tx = "0x".to_string() + &insc.mime_data[i..i + TRANSFER_TX_HEX_LENGTH];
            if let Some(item_insc) = self.db.read().unwrap().get_inscription_by_tx(&item_insc_tx) {
                if item_insc.mime_category != InscriptionMimeCategory::Text
                    || item_insc.mime_category != InscriptionMimeCategory::Image
                {
                    info!(
                        "[indexer] transfer inscription mime category not match: {} {}",
                        insc.tx_hash, item_insc_tx
                    );
                    return false;
                }

                let item_holder = self.get_nft_holder(item_insc.id);
                if item_holder == insc.from {
                    trans.push(NFTTransfer {
                        nft_id: item_insc.id,
                        transfer_id: insc.id,
                    });
                    match self.nft_holders.get_mut(&item_insc.id) {
                        Some(holder) => *holder = insc.to.clone(),
                        None => {
                            self.nft_holders.insert(item_insc.id, insc.to.clone());
                        }
                    }
                } else {
                    info!(
                        "[indexer] transfer inscription holder not match: {} {}",
                        insc.tx_hash, item_insc_tx
                    );
                    return false;
                }
            } else {
                info!("[indexer] transfer inscription not found: {} {}", insc.tx_hash, item_insc_tx);
                return false;
            }
        }

        self.nft_transfers.append(&mut trans);
        true
    }

    fn save_token_transfer(&self, txn: &rocksdb::Transaction<rocksdb::TransactionDB>) {
        for (tick, id) in &self.token_transfers {
            txn.inscription_token_transfer_insert(tick, *id);
        }
    }

    fn save_nft_transfer(&self, db: &TransactionDB, txn: &rocksdb::Transaction<rocksdb::TransactionDB>) {
        for trans in &self.nft_transfers {
            txn.inscription_nft_transfer_insert(trans);
        }

        for (insc_id, holder) in self.nft_holders.iter() {
            txn.inscription_nft_holder_update(db, *insc_id, holder);
        }
    }
}
