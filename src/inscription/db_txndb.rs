use super::db::*;
use super::types::*;
use crate::num_index;
use crate::txn_db::{DBBase, TxnDB};
use log::error;
use rocksdb::{DBAccess, Direction, IteratorMode};
use std::collections::HashMap;

impl<T: DBBase + TxnDB + DBAccess> InscribeDB for T {
    fn get_top_inscription_id(&self) -> u64 {
        self.get_u64(KEY_INSC_TOP)
    }

    fn get_top_inscription_sync_id(&self) -> u64 {
        self.get_u64(KEY_INSC_SYNC_TOP)
    }

    fn get_sync_blocknumber(&self) -> u64 {
        self.get_u64(KEY_SYNC_BLOCKNUMBER)
    }

    fn get_block_hash(&self, blocknumber: u64) -> Option<String> {
        let key = make_index_key(KEY_SYNC_BLOCK_HASH, blocknumber);
        self.get_string(&key)
    }

    fn inscription_sign_exists(&self, sign: &str) -> bool {
        let key = make_index_key(KEY_INSC_INDEX_SIGN, sign);
        self.get(key.as_bytes()).unwrap().is_some()
    }

    fn get_inscription_by_id(&self, id: u64) -> Option<Inscription> {
        let key_id = make_index_key(KEY_INSC_INDEX_ID, num_index!(id));
        match self.get(key_id.as_bytes()).unwrap() {
            Some(data) => {
                let mut insc: Inscription = serde_json::from_slice(&data).unwrap();
                if insc.mime_category == InscriptionMimeCategory::Json {
                    insc.json = serde_json::from_str::<serde_json::Value>(insc.mime_data.as_str()).unwrap();
                }
                Some(insc)
            }
            None => None,
        }
    }

    fn get_inscriptions_by_id(&self, id_list: &Vec<u64>) -> Vec<Inscription> {
        let mut insc_list: Vec<Inscription> = Vec::new();
        for id in id_list {
            if let Some(insc) = self.get_inscription_by_id(*id) {
                insc_list.push(insc);
            } else {
                error!("get_inscriptions_by_id Inscription not found: {}", id)
            }
        }
        insc_list
    }

    fn get_inscription_id_by_tx(&self, tx: &str) -> u64 {
        let key_tx = make_index_key(KEY_INSC_INDEX_TX, tx);
        self.get_u64(key_tx.as_str())
    }

    fn get_inscription_by_tx(&self, tx: &str) -> Option<Inscription> {
        let key_tx = make_index_key(KEY_INSC_INDEX_TX, tx);
        let id = self.get_u64(key_tx.as_str());
        self.get_inscription_by_id(id)
    }

    fn get_inscription_nft_holder_by_id(&self, id: u64) -> Option<String> {
        let key_id = make_index_key(KEY_INSC_NFT_INDEX_HOLDER, num_index!(id));
        self.get_string(&key_id)
    }

    fn get_first_value(&self, prefix: &str) -> Option<(Box<[u8]>, Box<[u8]>)> {
        let mut iter = self.iterator(IteratorMode::From(prefix.as_bytes(), Direction::Forward));
        match iter.next() {
            Some(Ok(item)) => {
                if item.0.starts_with(prefix.as_bytes()) {
                    Some(item)
                } else {
                    None
                }
            }
            Some(Err(_)) => None,
            None => None,
        }
    }

    fn get_token(&self, tick: &str) -> Option<InscriptionToken> {
        let key_tick = make_index_key(KEY_INSC_TOKEN_INDEX_TICK, tick);
        let id = self.get_u64(key_tick.as_str());
        if id == 0 {
            return None;
        }

        let key_id = make_index_key(KEY_INSC_TOKEN_INDEX_ID, num_index!(id));
        let result = self.get(key_id.as_bytes()).unwrap();
        serde_json::from_slice(&result.unwrap()).unwrap()
    }

    fn get_tokens(&self) -> HashMap<String, InscriptionToken> {
        let mut tokens = HashMap::new();
        let mut iter = self.iterator(IteratorMode::From(KEY_INSC_TOKEN_INDEX_ID.as_bytes(), Direction::Forward));

        while let Some(Ok((key, value))) = iter.next() {
            if !key.starts_with(KEY_INSC_TOKEN_INDEX_ID.as_bytes()) {
                break;
            }

            let token: InscriptionToken = serde_json::from_slice(&value).unwrap();
            tokens.insert(token.tick.to_string(), token);
        }

        tokens
    }

    fn get_tokens_list(&self) -> Vec<InscriptionToken> {
        let mut list = Vec::new();
        let mut iter = self.iterator(IteratorMode::From(KEY_INSC_TOKEN_INDEX_ID.as_bytes(), Direction::Forward));

        while let Some(Ok((key, value))) = iter.next() {
            if !key.starts_with(KEY_INSC_TOKEN_INDEX_ID.as_bytes()) {
                break;
            }

            let token: InscriptionToken = serde_json::from_slice(&value).unwrap();
            list.push(token);
        }
        list
    }

    fn token_exists_i(&self, tick: &str) -> bool {
        let key = make_index_key(KEY_INSC_TOKEN_INDEX_TICK_I, &tick.to_lowercase());
        self.get(key.as_bytes()).unwrap().is_some()
    }

    fn get_token_balance(&self, tick: &str, holder: &str) -> u64 {
        let key = make_index_key2(KEY_INSC_BALANCE_INDEX_HOLDER_TICK, holder, tick);
        self.get_u64(key.as_str())
    }

    fn get_items(&self, prefix: &str, start: &str, skip: u64, limit: u64, dir: rocksdb::Direction) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut item_list: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        let mut iter = self.iterator(IteratorMode::From(start.as_bytes(), dir));
        for _ in 0..skip {
            if let Some(Ok(_)) = iter.next() {
            } else {
                break;
            }
        }

        for _ in 0..limit {
            if let Some(Ok((key, value))) = iter.next() {
                if !key.starts_with(prefix.as_bytes()) {
                    break;
                }

                item_list.push((key.to_vec(), value.to_vec()));
            } else {
                break;
            }
        }

        item_list
    }

    fn get_item_keys(&self, prefix: &str, start: &str, skip: u64, limit: u64, dir: rocksdb::Direction) -> Vec<String> {
        let mut key_list: Vec<String> = Vec::new();
        let mut iter = self.iterator(IteratorMode::From(start.as_bytes(), dir));
        for _ in 0..skip {
            if let Some(Ok(_)) = iter.next() {
            } else {
                break;
            }
        }

        for _ in 0..limit {
            if let Some(Ok((key, _))) = iter.next() {
                if !key.starts_with(prefix.as_bytes()) {
                    break;
                }

                key_list.push(String::from_utf8(key.to_vec()).unwrap());
            } else {
                break;
            }
        }

        key_list
    }
}
