use super::{
    inscribe_collection::ProcessBlockContextJsonCollection,
    inscribe_token::ProcessBlockContextJsonToken,
    types::{InscribeContext, Inscription, APP_PROTO_COLLECTION},
};
use crate::config::TOKEN_PROTOCOL;
use log::debug;
use rocksdb::{Transaction, TransactionDB};

pub trait ProcessBlockContextJson {
    fn process_inscribe_json(&mut self, insc: &Inscription) -> bool;
    fn save_inscribe_json(&self, txn: &Transaction<TransactionDB>, insc: &Inscription);
}

impl<'a> ProcessBlockContextJson for InscribeContext<'a> {
    fn process_inscribe_json(&mut self, insc: &Inscription) -> bool {
        if let Some(protocol) = insc.json["p"].as_str() {
            if protocol == *TOKEN_PROTOCOL {
                self.execute_app_token(insc)
            } else if protocol == APP_PROTO_COLLECTION {
                self.execute_app_collection(insc)
            } else {
                debug!("[indexer] inscribe json: unknown protocol: {} {}", insc.tx_hash, protocol);
                false
            }
        } else {
            false
        }
    }

    fn save_inscribe_json(&self, txn: &Transaction<TransactionDB>, insc: &Inscription) {
        let p = insc.json["p"].as_str().unwrap();
        if p == APP_PROTO_COLLECTION {
            self.save_inscribe_collection(txn, insc);
        }
    }
}
