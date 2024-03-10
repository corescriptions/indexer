use super::types::{Inscription, InscriptionToken, NFTTransfer};
use crate::txn_db::TxnDB;
use rocksdb::TransactionDB;

pub const NUM_INDEX_LEN: usize = 18;
pub const DESC_INDEX_C: u64 = 10u64.pow(18);

#[macro_export]
macro_rules! num_index {
    ($num:expr) => {
        format!("{:018}", $num)
    };
}

#[macro_export]
macro_rules! num_index_desc {
    ($num:expr) => {
        format!("{:018}", 10u64.pow(18) - ($num))
    };
}

pub const KEY_ROLLBACK_BLOCKNUMBER: &'static str = "rollback_blocknumber";
pub const KEY_SYNC_BLOCKNUMBER: &'static str = "sync_blocknumber";
pub const KEY_SYNC_BLOCK_HASH: &'static str = "sync_blockhash";

// inscription top
pub const KEY_INSC_SYNC_TOP: &'static str = "insc_sync_top";
pub const KEY_INSC_TOP: &'static str = "insc_top";

// inscription index
pub const KEY_INSC_INDEX_ID: &'static str = "insc_id";
pub const KEY_INSC_INDEX_TX: &'static str = "insc_tx";
pub const KEY_INSC_INDEX_SIGN: &'static str = "insc_sign";
pub const KEY_INSC_INDEX_NFT_ID: &'static str = "insc_nft_id";
pub const KEY_INSC_INDEX_CREATER: &'static str = "insc_creater-id";
pub const KEY_INSC_INDEX_ADDRESS: &'static str = "insc_address-id";

// inscription nft
pub const KEY_INSC_NFT_INDEX_CREATER: &'static str = "insc_nft_creater-id";
pub const KEY_INSC_NFT_INDEX_HOLDER: &'static str = "insc_nft_holder_id";
pub const KEY_INSC_NFT_INDEX_HOLDER_ADDRESS: &'static str = "insc_nft_holder_address-id";

// inscription nft transfer
pub const KEY_INSC_NFT_TRANS_INDEX_ID: &'static str = "insc_nft_trans_id";

// inscription nft collection
pub const KEY_INSC_NFT_COLL_INDEX_ID: &'static str = "insc_coll_id";

// token
pub const KEY_INSC_TOKEN_INDEX_ID: &'static str = "insc_token_id";
pub const KEY_INSC_TOKEN_INDEX_TICK: &'static str = "insc_token_tick";
pub const KEY_INSC_TOKEN_INDEX_TICK_I: &'static str = "insc_token_itick";

// token transfer
pub const KEY_INSC_TOKEN_TRANSFER: &'static str = "insc_token_transfer_tick";

// token balance
pub const KEY_INSC_BALANCE_INDEX_TICK_BALANCE_HOLDER: &'static str = "insc_balance_tick_balance_holder";
pub const KEY_INSC_BALANCE_INDEX_HOLDER_TICK: &'static str = "insc_balance_holder_tick";

pub fn make_index_key<T: std::fmt::Display>(index: &str, key: T) -> String {
    format!("{}:{}", index, key)
}

pub fn make_index_key2<T1: std::fmt::Display, T2: std::fmt::Display>(index: &str, key1: T1, key2: T2) -> String {
    format!("{}:{}:{}", index, key1, key2)
}

pub fn make_index_key3<T1: std::fmt::Display, T2: std::fmt::Display, T3: std::fmt::Display>(
    index: &str,
    key1: T1,
    key2: T2,
    key3: T3,
) -> String {
    format!("{}:{}:{}:{}", index, key1, key2, key3)
}

pub trait InscribeDB: TxnDB {
    fn get_first_value(&self, prefix: &str) -> Option<(Box<[u8]>, Box<[u8]>)>;

    fn get_top_inscription_id(&self) -> u64;
    fn get_top_inscription_sync_id(&self) -> u64;
    fn get_sync_blocknumber(&self) -> u64;
    fn get_rollback_blocknumber(&self) -> u64;

    fn get_block_hash(&self, blocknumber: u64) -> Option<String>;

    fn get_inscription_by_id(&self, id: u64) -> Option<Inscription>;
    fn get_inscriptions_by_id(&self, id_list: &Vec<u64>) -> Vec<Inscription>;
    fn get_inscription_id_by_tx(&self, tx: &str) -> u64;
    fn get_inscription_by_tx(&self, tx: &str) -> Option<Inscription>;
    fn get_inscription_nft_holder_by_id(&self, id: u64) -> Option<String>;
    fn get_token(&self, tick: &str) -> Option<InscriptionToken>;
    fn get_tokens(&self) -> std::collections::HashMap<String, InscriptionToken>;
    fn get_tokens_list(&self) -> Vec<InscriptionToken>;
    fn token_exists_i(&self, tick: &str) -> bool;
    fn inscription_sign_exists(&self, sign: &str) -> bool;
    fn get_token_balance(&self, tick: &str, holder: &str) -> u64;
    fn get_items(&self, prefix: &str, start: &str, skip: u64, limit: u64, dir: rocksdb::Direction) -> Vec<(Vec<u8>, Vec<u8>)>;
    fn get_item_keys(&self, prefix: &str, start: &str, skip: u64, limit: u64, dir: rocksdb::Direction) -> Vec<String>;
}

pub trait InscribeTxn<'a> {
    fn set_top_inscription_id(&self, id: u64);
    fn set_top_inscription_sync_id(&self, id: u64);
    fn set_sync_blocknumber(&self, blocknumber: u64);
    fn set_rollback_blocknumber(&self, blocknumber: u64);
    fn set_block_hash(&self, blocknumber: u64, block_hash: &str);

    fn inscription_insert(&self, insc: &Inscription);
    fn inscription_inscribe(&self, insc: &Inscription);
    fn inscription_update(&self, insc: &Inscription);
    fn inscription_nft_holder_update(&self, db: &TransactionDB, id: u64, new_holder: &str);
    fn inscription_nft_transfer_insert(&self, trans: &NFTTransfer);
    fn inscription_nft_collection_insert(&self, insc: &Inscription);
    fn inscription_token_insert(&self, token: &InscriptionToken);
    fn inscription_token_transfer_insert(&self, tick: &str, id: u64);
    fn inscription_token_update(&self, token: &InscriptionToken);
    fn inscription_token_banalce_update(&self, db: &TransactionDB, tick: &str, holder: &str, balance_change: i64) -> i64;
    fn delete_keys(&self, prefix: &str, max: u64) -> u64;
}

pub fn db_index2str(keys: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for item in &keys {
        item.rfind(':').map(|i| result.push(item[i + 1..].to_string()));
    }
    result
}

pub fn db_key_index2id(key: &str) -> u64 {
    key[key.len() - NUM_INDEX_LEN..].parse::<u64>().unwrap()
}

pub fn db_key_index2id_desc(key: &str) -> u64 {
    DESC_INDEX_C - key[key.len() - NUM_INDEX_LEN..].parse::<u64>().unwrap()
}

pub fn db_index2id(keys: Vec<String>) -> Vec<u64> {
    let mut result = Vec::new();
    for item in keys {
        result.push(db_key_index2id(&item));
    }
    result
}

pub fn db_index2id_desc(keys: Vec<String>) -> Vec<u64> {
    let mut result = Vec::new();
    for item in keys {
        result.push(db_key_index2id_desc(&item));
    }
    result
}

pub fn db_val2u64(v: Vec<Vec<u8>>) -> Vec<u64> {
    let mut result = Vec::new();
    for item in v {
        let ptr = item.as_ptr() as *const u64;
        result.push(unsafe { u64::from_be(*ptr) });
    }
    result
}

pub fn db_val2string(v: Vec<Vec<u8>>) -> Vec<String> {
    let mut result = Vec::new();
    for item in v {
        result.push(String::from_utf8(item).unwrap());
    }
    result
}

pub fn db_val2json<T: serde::de::DeserializeOwned>(v: Vec<Vec<u8>>) -> Vec<T> {
    let mut result = Vec::new();
    for item in v {
        result.push(serde_json::from_slice(&item).unwrap());
    }
    result
}

pub fn db_item_val2json<T: serde::de::DeserializeOwned>(v: Vec<(Vec<u8>, Vec<u8>)>) -> Vec<T> {
    let mut result = Vec::new();
    for item in v {
        result.push(serde_json::from_slice(&item.1).unwrap());
    }
    result
}
