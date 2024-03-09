pub mod event_log;
pub mod web3ex;

use async_trait::async_trait;
use std::collections::HashMap;
use web3::{
    types::{Block, Transaction, H256},
    Transport,
};

#[async_trait]
pub trait Web3Ex<T: Transport + Send + Sync> {
    async fn get_chain_id(&self) -> u64;
    async fn get_blocknumber(&self) -> Option<u64>;
    async fn get_blocknumber_wait(&self) -> u64;
    async fn get_block_with_txs(&self, blocknumber: u64) -> Option<Block<Transaction>>;
    async fn get_block_wait(&self, blocknumber: u64) -> Block<Transaction>;
    async fn get_block_info(&self, blocknumber: u64) -> Option<Block<H256>>;
    async fn get_block_info_wait(&self, blocknumber: u64) -> Block<H256>;
    async fn get_event_logs(&self, contracts: &Vec<String>, blocknumber: u64) -> Option<Vec<web3::types::Log>>;
    async fn get_event_logs_wait(&self, contracts: &Vec<String>, blocknumber: u64) -> HashMap<u64, Vec<web3::types::Log>>;
}

pub trait Web3LogEvent {
    fn match_event(&self, contract: &web3::ethabi::Contract, event_name: &str) -> Option<web3::ethabi::Log>;
}

pub trait Web3ABILogEvent {
    fn get_param(&self, name: &str) -> Option<&web3::ethabi::Token>;
}

pub trait HexParseTrait {
    fn to_hex_string(&self) -> String;
}

pub fn init_web3_http(url: &str) -> web3::Web3<web3::transports::Http> {
    assert!(url.starts_with("http"));
    let transport = web3::transports::Http::new(url).unwrap();
    let web3 = web3::Web3::new(transport);
    web3
}
