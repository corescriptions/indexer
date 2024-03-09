use std::sync::Mutex;

lazy_static! {
    pub static ref ROLLBACK_BLOCK: Mutex<u64> = Mutex::new(0);
}

pub async fn sleep_ms(ms: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
}

pub fn get_timestamp_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
