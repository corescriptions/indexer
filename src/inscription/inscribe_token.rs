use super::{
    db::InscribeDB,
    marketplace::APP_OPER_TOKEN_MARKET_LIST,
    trait_json_value::JsonValueTrait,
    types::{InscribeContext, Inscription, InscriptionToken},
};
use crate::config::{MARKET_ADDRESS_LIST, START_BLOCK_MINT, TICK_MAX_LEN};
use log::{debug, info};
use std::collections::HashMap;

const APP_OPER_TOKEN_DEPLOY: &'static str = "deploy";
const APP_OPER_TOKEN_MINT: &'static str = "mint";
const APP_OPER_TOKEN_TRANSFER: &'static str = "transfer";

const TOKEN_BALANCE_MAX: u64 = 1e18 as u64;

pub trait ProcessBlockContextJsonToken {
    fn check_deploy(&self, insc: &Inscription) -> bool;
    fn check_mint(&self, insc: &Inscription) -> bool;
    fn check_transfer(&self, insc: &Inscription) -> bool;

    fn get_token_balance(&self, tick: &str, address: &str) -> u64;
    fn token_balance_change_update(&mut self, tick: &str, address: &str, amount: i64);

    fn execute_app_token(&mut self, insc: &Inscription) -> bool;
    fn execute_app_token_deploy(&mut self, insc: &Inscription) -> bool;
    fn execute_app_token_mint(&mut self, insc: &Inscription) -> bool;
    fn execute_app_token_transfer(&mut self, insc: &Inscription) -> bool;
    fn execute_app_token_market_list(&mut self, insc: &Inscription) -> bool;
}

impl<'a> ProcessBlockContextJsonToken for InscribeContext<'a> {
    fn execute_app_token(&mut self, insc: &Inscription) -> bool {
        let op_value = &insc.json["op"];
        if !op_value.is_string() {
            info!("[indexer] inscribe token invalid oper {}", insc.tx_hash);
            return false;
        }

        if !insc.json["tick"].is_string() {
            info!("[indexer] inscribe token invalid tick {}", insc.tx_hash);
            return false;
        }

        let op = op_value.as_str().unwrap();

        match op {
            APP_OPER_TOKEN_DEPLOY => self.execute_app_token_deploy(insc),
            APP_OPER_TOKEN_MINT => self.execute_app_token_mint(insc),
            APP_OPER_TOKEN_TRANSFER => self.execute_app_token_transfer(insc),
            APP_OPER_TOKEN_MARKET_LIST => self.execute_app_token_market_list(insc),
            _ => false,
        }
    }

    fn check_deploy(&self, insc: &Inscription) -> bool {
        let tick = insc.json["tick"].as_str().unwrap();
        if tick.len() > *TICK_MAX_LEN {
            return false;
        }

        if tick.find(':').is_some() {
            return false;
        }

        let token_max = match insc.json["max"].parse_u64() {
            Some(value) => value,
            None => {
                return false;
            }
        };

        let token_lmi = match insc.json["lmi"].parse_u64() {
            Some(value) => value,
            None => {
                return false;
            }
        };

        if token_max > TOKEN_BALANCE_MAX || token_lmi > token_max {
            return false;
        }

        if self.db.read().unwrap().token_exists_i(tick) {
            info!("[indexer] inscribe token deploy: token existed: {} {}", insc.tx_hash, tick);
            return false;
        }

        true
    }

    fn execute_app_token_deploy(&mut self, insc: &Inscription) -> bool {
        if !self.check_deploy(insc) {
            return false;
        }

        let tick = insc.json["tick"].as_str().unwrap();
        let token_max = insc.json["max"].parse_u64().unwrap();
        let token_lmi = insc.json["lmi"].parse_u64().unwrap();

        self.token_cache.insert(
            tick.to_string(),
            InscriptionToken {
                insc_id: insc.id,
                tick: tick.to_string(),
                tick_i: tick.to_lowercase(),
                tx: insc.tx_hash.to_string(),
                from: insc.from.to_string(),
                blocknumber: insc.blocknumber,
                timestamp: insc.timestamp,
                holders: 0,
                mint_max: token_max,
                mint_limit: token_lmi,
                mint_progress: 0,
                mint_finished: false,
                updated: true,
                deploy: true,

                market_updated: false,
                market_volume24h: 0,
                market_txs24h: 0,
                market_cap: 0,
                market_floor_price: 0,
            },
        );

        info!("[indexer] inscribe token deploy: {} {}", insc.tx_hash, tick);
        true
    }

    fn check_mint(&self, insc: &Inscription) -> bool {
        if *START_BLOCK_MINT > insc.blocknumber {
            return false;
        }

        let tick = insc.json["tick"].as_str().unwrap();
        let mint_amt = match insc.json["amt"].parse_u64() {
            Some(value) => value,
            None => {
                debug!("[indexer] token mint: invalid amount: {} {}", insc.tx_hash, tick);
                return false;
            }
        };

        if mint_amt == 0 || mint_amt > TOKEN_BALANCE_MAX {
            debug!("[indexer] token mint: invalid amount: {} {} {}", insc.tx_hash, tick, mint_amt);
            return false;
        }

        let token = match self.token_cache.get(tick) {
            Some(value) => value,
            None => {
                debug!("[indexer] token mint: token not found: {} {}", insc.tx_hash, tick);
                return false;
            }
        };

        if mint_amt > token.mint_limit {
            debug!("[indexer] token mint: mint limit: {} {}", insc.tx_hash, tick);
            return false;
        }

        if self.inscribe_patch.mint_pass_tx.contains(&insc.tx_hash) {
            return true;
        }

        if token.mint_finished {
            debug!("[indexer] token mint: mint finished: {} {}", insc.tx_hash, tick);
            return false;
        }

        if mint_amt + token.mint_progress > token.mint_max {
            debug!("[indexer] token mint: mint overflow: {} {}", insc.tx_hash, tick);
            return false;
        }

        true
    }

    fn check_transfer(&self, insc: &Inscription) -> bool {
        let tick = insc.json["tick"].as_str().unwrap();
        let transfer_amt = match insc.json["amt"].parse_u64() {
            Some(value) => value,
            None => {
                info!("[indexer] token transfer: invalid amount: {} {}", insc.tx_hash, tick);
                return false;
            }
        };

        if transfer_amt == 0 || transfer_amt > TOKEN_BALANCE_MAX {
            info!(
                "[indexer] token transfer: invalid amount: {} {} {}",
                insc.tx_hash, tick, transfer_amt
            );
            return false;
        }

        let token = match self.token_cache.get(tick) {
            Some(value) => value,
            None => {
                info!("[indexer] token transfer: token not found: {} {}", insc.tx_hash, tick);
                return false;
            }
        };

        if !token.mint_finished {
            info!("[indexer] token transfer: mint not finished: {} {}", insc.tx_hash, tick);
            return false;
        }

        true
    }

    fn execute_app_token_mint(&mut self, insc: &Inscription) -> bool {
        if !self.check_mint(insc) {
            return false;
        }

        let tick = insc.json["tick"].as_str().unwrap();
        let mint_amt = insc.json["amt"].parse_u64().unwrap();
        self.token_balance_change_update(tick, &insc.to, mint_amt as i64);

        let token_cache = self.token_cache.get_mut(tick).unwrap();
        token_cache.mint_progress += mint_amt;
        if token_cache.mint_progress >= token_cache.mint_max {
            token_cache.mint_finished = true;
        }

        debug!("[indexer] inscribe token mint: {} {}", insc.tx_hash, tick);
        true
    }

    fn token_balance_change_update(&mut self, tick: &str, address: &str, amount: i64) {
        let balance_change_coll = self.token_balance_change.get_mut(tick);
        let balance_change_coll = match balance_change_coll {
            Some(value) => value,
            None => {
                self.token_balance_change.insert(tick.to_string(), HashMap::new());
                self.token_balance_change.get_mut(tick).unwrap()
            }
        };

        let balance_change_value = balance_change_coll.get_mut(address);
        match balance_change_value {
            Some(value) => {
                *value += amount;
            }
            None => {
                balance_change_coll.insert(address.to_string(), amount);
            }
        }
    }

    fn get_token_balance(&self, tick: &str, address: &str) -> u64 {
        let balance = self.db.read().unwrap().get_token_balance(tick, address);
        let balance_change = match self.token_balance_change.get(tick) {
            Some(value) => value.get(address),
            None => None,
        };
        if let Some(balance_change) = balance_change {
            (balance as i64 + *balance_change) as u64
        } else {
            balance
        }
    }

    fn execute_app_token_transfer(&mut self, insc: &Inscription) -> bool {
        if !self.check_transfer(insc) {
            return false;
        }

        let tick = insc.json["tick"].as_str().unwrap();
        let transfer_amount = insc.json["amt"].parse_u64().unwrap();
        let balance_from = self.get_token_balance(tick, &insc.from);

        if transfer_amount <= balance_from {
            self.token_balance_change_update(tick, &insc.from, -(transfer_amount as i64));
            self.token_balance_change_update(tick, &insc.to, transfer_amount as i64);
            self.token_transfers.push((tick.to_string(), insc.id));
            info!(
                "[indexer] token transfer: {} {} {} {} {} {}",
                insc.tx_hash, insc.from, insc.to, tick, balance_from, transfer_amount
            );
            true
        } else {
            info!(
                "[indexer] token transfer failed: {} {} {} {} {} {} ",
                insc.tx_hash, insc.from, insc.to, tick, balance_from, transfer_amount
            );
            false
        }
    }

    fn execute_app_token_market_list(&mut self, insc: &Inscription) -> bool {
        if !MARKET_ADDRESS_LIST.contains(&insc.to) {
            info!(
                "[indexer] token market list: invalid market address: {} {} {}",
                insc.tx_hash, insc.from, insc.to
            );
            return false;
        }
        info!("[indexer] token market list: {} {} {}", insc.tx_hash, insc.from, insc.to);
        self.execute_app_token_transfer(insc)
    }
}
