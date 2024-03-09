use std::collections::HashMap;

use super::{HexParseTrait, Web3Ex};
use crate::global::sleep_ms;
use async_trait::async_trait;
use log::warn;
use web3::{
    types::{Address, Block, BlockId, FilterBuilder, Log, Transaction, H256},
    Transport, Web3,
};

#[async_trait]
impl<T: Transport + Send + Sync> Web3Ex<T> for Web3<T>
where
    T::Out: Send,
{
    async fn get_chain_id(&self) -> u64 {
        self.eth().chain_id().await.unwrap().as_u64()
    }

    async fn get_blocknumber(&self) -> Option<u64> {
        match self.eth().block_number().await {
            Ok(blocknumber) => Some(blocknumber.as_u64()),
            Err(e) => {
                warn!("[web3] get_blocknumber error: {}", e.to_string());
                None
            }
        }
    }

    async fn get_blocknumber_wait(&self) -> u64 {
        loop {
            if let Some(blocknumber) = self.get_blocknumber().await {
                return blocknumber;
            } else {
                sleep_ms(1000).await;
            }
        }
    }

    async fn get_block_with_txs(&self, blocknumber: u64) -> Option<Block<Transaction>> {
        match self.eth().block_with_txs(BlockId::Number(blocknumber.into())).await {
            Ok(block) => block,
            Err(e) => {
                warn!("[web3] get_block error: {} {}", blocknumber, e.to_string());
                None
            }
        }
    }

    async fn get_block_wait(&self, blocknumber: u64) -> Block<Transaction> {
        loop {
            if let Some(block) = self.get_block_with_txs(blocknumber).await {
                return block;
            } else {
                sleep_ms(1000).await;
            }
        }
    }

    async fn get_block_info(&self, blocknumber: u64) -> Option<Block<H256>> {
        match self.eth().block(BlockId::Number(blocknumber.into())).await {
            Ok(block) => block,
            Err(e) => {
                warn!("[web3] get_block error: {} {}", blocknumber, e.to_string());
                None
            }
        }
    }

    async fn get_block_info_wait(&self, blocknumber: u64) -> Block<H256> {
        loop {
            if let Some(block) = self.get_block_info(blocknumber).await {
                return block;
            } else {
                sleep_ms(1000).await;
            }
        }
    }

    async fn get_event_logs(&self, contracts: &Vec<String>, blocknumber: u64) -> Option<Vec<Log>> {
        let filter = FilterBuilder::default()
            .address(contracts.iter().map(|x| x.parse().unwrap()).collect())
            .from_block(blocknumber.into())
            .to_block(blocknumber.into())
            .build();

        match self.eth().logs(filter).await {
            Ok(logs) => Some(logs),
            Err(e) => {
                warn!(
                    "[web3] get_event_logs error: {} {} {}",
                    contracts.join(","),
                    blocknumber,
                    e.to_string()
                );
                None
            }
        }
    }

    async fn get_event_logs_wait(&self, contracts: &Vec<String>, blocknumber: u64) -> HashMap<u64, Vec<web3::types::Log>> {
        let mut block_logs: HashMap<u64, Vec<web3::types::Log>> = HashMap::new();
        let logs;

        loop {
            if let Some(_logs) = self.get_event_logs(contracts, blocknumber).await {
                logs = _logs;
                break;
            }
            sleep_ms(1000).await;
        }

        for log in &logs {
            let tx_index = log.transaction_index.unwrap().as_u64();
            match block_logs.get_mut(&tx_index) {
                Some(tx_logs) => tx_logs.push(log.clone()),
                None => {
                    let tx_logs = vec![log.clone()];
                    block_logs.insert(tx_index, tx_logs);
                }
            };
        }

        block_logs
    }
}

impl HexParseTrait for H256 {
    fn to_hex_string(&self) -> String {
        format!("{:#x}", self)
    }
}

impl HexParseTrait for Address {
    fn to_hex_string(&self) -> String {
        format!("{:#x}", self)
    }
}
