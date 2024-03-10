use super::types::*;
use crate::{config::MARKET_ADDRESS_LIST, ethereum::HexParseTrait};
use log::debug;
use regex::Regex;
use web3::types::{Block, Transaction};

lazy_static! {
    pub static ref DATA_REGEX: Regex = Regex::new(r"^data:(.*?),(.+)$").unwrap();
}

pub trait TrailsTx {
    fn to_inscription(&self, block: &Block<Transaction>, logs: Option<&Vec<web3::types::Log>>, id: u64) -> Option<Inscription>;
    fn inscription_check(&self) -> bool;
    fn inscription_prepare(&self, insc: &mut Inscription, logs: Option<&Vec<web3::types::Log>>) -> bool;
    fn inscription_get_mimecategory_plain(&self, mime_type: &str) -> InscriptionMimeCategory;
    fn inscription_is_json_object(&self, mime_type: &str, mime_data: &str) -> bool;
    fn get_order_id(&self) -> String;
}

impl TrailsTx for Transaction {
    fn inscription_check(&self) -> bool {
        if self.to.is_none() {
            return false; // deploy smart contract
        }

        true
    }

    fn get_order_id(&self) -> String {
        let address: web3::types::Address = self.from.unwrap().into();
        let blocknumber = self.block_number.unwrap().as_u64();
        let blocknumber_u256 = web3::types::U256::from(blocknumber);
        let blocknumber_bytes: [u8; 32] = blocknumber_u256.into();

        let mut data = Vec::from(address.as_bytes());
        data.extend(blocknumber_bytes);
        data.extend(self.input.0.clone());

        let encoded_data = web3::signing::keccak256(&data);

        "0x".to_string() + hex::encode(encoded_data).as_str()
    }

    fn to_inscription(&self, block: &Block<Transaction>, logs: Option<&Vec<web3::types::Log>>, id: u64) -> Option<Inscription> {
        if !self.inscription_check() {
            return None;
        }

        let mut insc = Inscription {
            id,
            tx_hash: self.hash.to_hex_string(),
            tx_index: self.transaction_index.unwrap().as_u64(),
            blocknumber: block.number.unwrap().as_u64(),
            from: self.from.unwrap().to_hex_string().to_lowercase(),
            to: self.to.unwrap().to_hex_string().to_lowercase(),
            mime_type: "".to_string(),
            mime_data: "".to_string(),
            mime_category: InscriptionMimeCategory::Null,
            signature: None,
            timestamp: block.timestamp.as_u64(),
            verified: InscriptionVerifiedStatus::Unresolved,
            event_logs: Vec::new(),
            market_order_id: None,

            json: serde_json::Value::Null,
        };

        if MARKET_ADDRESS_LIST.contains(&insc.to) {
            insc.market_order_id = Some(self.get_order_id());
        }

        if self.inscription_prepare(&mut insc, logs) {
            Some(insc)
        } else {
            None
        }
    }

    fn inscription_is_json_object(&self, mime_type: &str, mime_data: &str) -> bool {
        if mime_type.is_empty() || mime_type == "application/json" {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(mime_data) {
                return json.is_object();
            }
        }

        false
    }

    fn inscription_get_mimecategory_plain(&self, mime_type: &str) -> InscriptionMimeCategory {
        if mime_type.starts_with("image/") {
            InscriptionMimeCategory::Image
        } else if mime_type == "" || mime_type.starts_with("text/") {
            InscriptionMimeCategory::Text
        } else {
            InscriptionMimeCategory::Null
        }
    }

    fn inscription_prepare(&self, insc: &mut Inscription, logs: Option<&Vec<web3::types::Log>>) -> bool {
        if logs.is_some() && logs.unwrap().len() > 0 {
            insc.event_logs = logs.unwrap().clone();
        }

        if let Ok(utf8_str) = std::str::from_utf8(&self.input.0) {
            if let Some(captures) = DATA_REGEX.captures(utf8_str) {
                let mime_type = captures.get(1).unwrap().as_str();
                let mime_data = captures.get(2).unwrap().as_str();
                let is_json = self.inscription_is_json_object(mime_type, mime_data);
                let mime_category = if is_json {
                    InscriptionMimeCategory::Json
                } else {
                    self.inscription_get_mimecategory_plain(mime_type)
                };

                if mime_category == InscriptionMimeCategory::Null {
                    debug!("[indexer] inscribe invalid mime category: {}", insc.tx_hash.as_str());
                    return false;
                }

                insc.mime_category = mime_category;
                insc.mime_type = mime_type.to_string();
                insc.mime_data = mime_data.to_string();

                true
            } else {
                false
            }
        } else if logs.is_some() && logs.unwrap().len() > 0 {
            insc.mime_category = InscriptionMimeCategory::Invoke;
            true
        } else if self.input.0.len() % TRANSFER_TX_RAW_LENGTH == 0 {
            insc.mime_category = InscriptionMimeCategory::Transfer;
            insc.mime_data = hex::encode(&self.input.0).to_string();
            true
        } else {
            false
        }
    }
}
