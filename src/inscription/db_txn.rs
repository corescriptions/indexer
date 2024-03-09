use super::db::*;
use super::types::*;
use crate::num_index;
use crate::num_index_desc;
use crate::txn_db::TxnDB;
use log::{debug, info};
use rocksdb::{Transaction, TransactionDB};

impl<'a> InscribeTxn<'a> for Transaction<'a, TransactionDB> {
    fn set_top_inscription_id(&self, id: u64) {
        self.put(KEY_INSC_TOP.as_bytes(), id.to_be_bytes()).unwrap();
    }

    fn set_top_inscription_sync_id(&self, id: u64) {
        self.put(KEY_INSC_SYNC_TOP.as_bytes(), id.to_be_bytes()).unwrap();
    }

    fn set_sync_blocknumber(&self, blocknumber: u64) {
        self.put(KEY_SYNC_BLOCKNUMBER.as_bytes(), blocknumber.to_be_bytes()).unwrap();
    }

    fn set_rollback_blocknumber(&self, blocknumber: u64) {
        self.put(KEY_ROLLBACK_BLOCKNUMBER.as_bytes(), blocknumber.to_be_bytes()).unwrap();
    }

    fn set_block_hash(&self, blocknumber: u64, block_hash: &str) {
        let key = make_index_key(KEY_SYNC_BLOCK_HASH, blocknumber);
        self.put(key.as_bytes(), block_hash.as_bytes()).unwrap();
    }

    fn inscription_insert(&self, insc: &Inscription) {
        self.inscription_update(insc);

        let index_key_tx = make_index_key(KEY_INSC_INDEX_TX, &insc.tx_hash);
        let index_key_creater = make_index_key2(KEY_INSC_INDEX_CREATER, &insc.from, num_index_desc!(insc.id));

        self.put(index_key_tx.as_bytes(), insc.id.to_be_bytes()).unwrap();
        self.put(index_key_creater.as_bytes(), "").unwrap();
    }

    fn inscription_inscribe(&self, insc: &Inscription) {
        self.inscription_update(insc);

        if insc.verified == InscriptionVerifiedStatus::Successful {
            let index_key_from = make_index_key2(KEY_INSC_INDEX_ADDRESS, &insc.from, num_index_desc!(insc.id));
            let index_key_to = make_index_key2(KEY_INSC_INDEX_ADDRESS, &insc.to, num_index_desc!(insc.id));

            self.put(index_key_from.as_bytes(), "").unwrap();
            self.put(index_key_to.as_bytes(), "").unwrap();

            // nft inscription
            if insc.signature.is_some() {
                let index_key_nft_id = make_index_key(KEY_INSC_INDEX_NFT_ID, num_index_desc!(insc.id));
                let index_key_sign = make_index_key(KEY_INSC_INDEX_SIGN, &insc.signature.clone().unwrap());
                let index_key_nft_creater = make_index_key2(KEY_INSC_NFT_INDEX_CREATER, &insc.from, num_index_desc!(insc.id));
                let index_key_nft_holder = make_index_key(KEY_INSC_NFT_INDEX_HOLDER, num_index!(insc.id));
                let index_key_nft_holder_address =
                    make_index_key2(KEY_INSC_NFT_INDEX_HOLDER_ADDRESS, &insc.to, num_index_desc!(insc.id));

                self.put(index_key_nft_id.as_bytes(), "").unwrap();
                self.put(index_key_sign.as_bytes(), insc.id.to_be_bytes()).unwrap();
                self.put(index_key_nft_creater.as_bytes(), "").unwrap();
                self.put(index_key_nft_holder.as_bytes(), insc.to.as_bytes()).unwrap();
                self.put(index_key_nft_holder_address.as_bytes(), "").unwrap();
            }
        }
    }

    fn inscription_update(&self, insc: &Inscription) {
        let insc_data = serde_json::to_string(insc).unwrap();
        let index_key_id = make_index_key(KEY_INSC_INDEX_ID, num_index!(insc.id));
        self.put(index_key_id.as_bytes(), insc_data.as_bytes()).unwrap();
    }

    fn inscription_nft_holder_update(&self, db: &TransactionDB, id: u64, new_holder: &str) {
        let index_key_nft_holder_id = make_index_key(KEY_INSC_NFT_INDEX_HOLDER, num_index!(id));
        let old_holder = db.get_string(&index_key_nft_holder_id).unwrap();

        let old_key_holder = make_index_key2(KEY_INSC_NFT_INDEX_HOLDER_ADDRESS, &old_holder, num_index_desc!(id));
        let new_key_holder = make_index_key2(KEY_INSC_NFT_INDEX_HOLDER_ADDRESS, new_holder, num_index_desc!(id));

        self.delete(old_key_holder.as_bytes()).unwrap();

        self.put(index_key_nft_holder_id.as_bytes(), new_holder.as_bytes()).unwrap();
        self.put(new_key_holder.as_bytes(), "").unwrap();
    }

    fn inscription_nft_transfer_insert(&self, insc_id: u64, transfer_insc_id: u64, index: u64) {
        let index_key_id = make_index_key3(
            KEY_INSC_NFT_TRANS_INDEX_ID,
            num_index!(insc_id),
            num_index!(transfer_insc_id),
            num_index!(index),
        );

        self.put(index_key_id.as_bytes(), "").unwrap();
    }

    fn inscription_nft_collection_insert(&self, insc: &Inscription) {
        let index_key_id = make_index_key(KEY_INSC_NFT_COLL_INDEX_ID, num_index_desc!(insc.id));
        self.put(index_key_id.as_bytes(), insc.id.to_be_bytes()).unwrap();
    }

    fn inscription_token_insert(&self, token: &InscriptionToken) {
        let json_data = serde_json::to_string(token).unwrap();
        let index_key_id = make_index_key(KEY_INSC_TOKEN_INDEX_ID, num_index!(token.insc_id));
        let index_key_tick = make_index_key(KEY_INSC_TOKEN_INDEX_TICK, &token.tick);
        let index_key_tick_i = make_index_key(KEY_INSC_TOKEN_INDEX_TICK_I, &token.tick_i);

        self.put(index_key_id.as_bytes(), json_data.as_bytes()).unwrap();
        self.put(index_key_tick.as_bytes(), token.insc_id.to_be_bytes()).unwrap();
        self.put(index_key_tick_i.as_bytes(), token.insc_id.to_be_bytes()).unwrap();
    }

    fn inscription_token_transfer_insert(&self, tick: &str, id: u64) {
        let index_key_transfer = make_index_key2(KEY_INSC_TOKEN_TRANSFER, tick, num_index_desc!(id));
        self.put(index_key_transfer.as_bytes(), "").unwrap();
    }

    fn inscription_token_banalce_update(&self, db: &TransactionDB, tick: &str, holder: &str, balance_change: i64) -> i64 {
        let old_balance = db.get_token_balance(tick, holder);
        if old_balance > 0 {
            let old_key_tick_balance_holder = make_index_key3(
                KEY_INSC_BALANCE_INDEX_TICK_BALANCE_HOLDER,
                tick,
                num_index_desc!(old_balance),
                holder,
            );
            let old_key_holder_tick = make_index_key2(KEY_INSC_BALANCE_INDEX_HOLDER_TICK, holder, tick);

            self.delete(old_key_tick_balance_holder.as_bytes()).unwrap();
            self.delete(old_key_holder_tick.as_bytes()).unwrap();
        }

        assert!(
            old_balance as i64 + balance_change >= 0,
            "invalid balance change {} {} {} {}",
            tick,
            holder,
            old_balance,
            balance_change
        );

        let new_balance = (old_balance as i64 + balance_change) as u64;
        if new_balance > 0 {
            let new_key_tick_holder = make_index_key3(
                KEY_INSC_BALANCE_INDEX_TICK_BALANCE_HOLDER,
                tick,
                num_index_desc!(new_balance),
                holder,
            );
            let new_key_holder_tick = make_index_key2(KEY_INSC_BALANCE_INDEX_HOLDER_TICK, holder, tick);

            self.put(new_key_tick_holder.as_bytes(), new_balance.to_be_bytes()).unwrap();
            self.put(new_key_holder_tick.as_bytes(), new_balance.to_be_bytes()).unwrap();
        }

        debug!(
            "[indexer] balance update: {} {} {} {}",
            tick, holder, old_balance, new_balance
        );

        return if old_balance > 0 && new_balance > 0 {
            0
        } else if old_balance == 0 && new_balance > 0 {
            1
        } else if new_balance == 0 && old_balance > 0 {
            -1
        } else {
            panic!("invalid balance change");
        };
    }

    fn inscription_token_update(&self, token: &InscriptionToken) {
        let index_key_id = make_index_key(KEY_INSC_TOKEN_INDEX_ID, num_index!(token.insc_id));
        let token_data = serde_json::to_string(token).unwrap();
        self.put(index_key_id.as_bytes(), token_data.as_bytes()).unwrap();
    }

    fn delete_keys(&self, prefix: &str, max: u64) -> u64 {
        let mut count = 0;
        let mut iter = self.prefix_iterator(prefix.as_bytes());
        while let Some(Ok((key, _))) = iter.next() {
            if !key.starts_with(prefix.as_bytes()) {
                break;
            }

            count += 1;
            self.delete(key).unwrap();
            if count >= max {
                break;
            }
        }

        info!("delete keys: {} {}", prefix, count);
        count
    }
}
