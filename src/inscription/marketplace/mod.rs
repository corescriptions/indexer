pub mod db;
pub mod inscribe_market;
pub mod market_abi;
pub mod types;

use self::types::MarketOrder;
use super::types::{Inscription, InscriptionToken};
use rocksdb::{Transaction, TransactionDB};

pub const APP_OPER_TOKEN_MARKET_LIST: &'static str = "market_list";

pub trait MarketPlace {
    fn process_inscribe_invoke(&mut self, insc: &mut Inscription) -> bool;

    fn execute_market_buy(&mut self, insc: &Inscription, log: &web3::ethabi::Log) -> bool;
    fn execute_market_buy_token(&mut self, insc: &Inscription, order: &MarketOrder, log: &web3::ethabi::Log) -> bool;
    fn execute_market_buy_nft(&mut self, insc: &Inscription, order: &MarketOrder, log: &web3::ethabi::Log) -> bool;

    fn execute_market_cancel(&mut self, insc: &Inscription, log: &web3::ethabi::Log) -> bool;
    fn execute_market_cancel_token(&mut self, insc: &Inscription, order: &MarketOrder) -> bool;
    fn execute_market_cancel_nft(&mut self, insc: &Inscription, order: &MarketOrder) -> bool;

    fn execute_market_set_price(&mut self, insc: &Inscription, log: &web3::ethabi::Log) -> bool;

    fn save_market(&self, db: &TransactionDB, txn: &Transaction<TransactionDB>, insc: &Inscription);
    fn save_market_new_order_token(&self, txn: &Transaction<TransactionDB>, insc: &Inscription);
    fn save_market_new_order_nft(&self, txn: &Transaction<TransactionDB>, insc: &Inscription);
    fn save_market_set_price(
        &self,
        db: &TransactionDB,
        txn: &Transaction<TransactionDB>,
        insc: &Inscription,
        log: &web3::ethabi::Log,
    );
    fn save_market_buy(
        &self,
        db: &TransactionDB,
        txn: &Transaction<TransactionDB>,
        insc: &Inscription,
        log: &web3::ethabi::Log,
    );
    fn save_market_cancel(
        &self,
        db: &TransactionDB,
        txn: &Transaction<TransactionDB>,
        insc: &Inscription,
        log: &web3::ethabi::Log,
    );

    fn update_token_market_info(db: &TransactionDB, token: &mut InscriptionToken);
}
