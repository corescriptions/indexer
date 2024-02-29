pub const DEFAULT_DB_PATH: &'static str = "./data";

lazy_static! {
    pub static ref WORKER_COUNT: u64 = std::env::var("WORKER_COUNT")
        .expect("WORKER_COUNT must be set")
        .parse::<u64>()
        .unwrap();
    pub static ref CONFIRM_BLOCK: u64 = std::env::var("CONFIRM_BLOCK")
        .expect("CONFIRM_BLOCK must be set")
        .parse::<u64>()
        .unwrap();
    pub static ref CHAIN_ID: u64 = std::env::var("CHAIN_ID")
        .expect("CHAIN_ID must be set")
        .parse::<u64>()
        .unwrap();
    pub static ref WEB3_PROVIDER: String = std::env::var("WEB3_PROVIDER")
        .expect("WEB3_PROVIDER must be set")
        .parse::<String>()
        .unwrap();
    pub static ref START_BLOCK: u64 = std::env::var("START_BLOCK")
        .expect("START_BLOCK must be set")
        .parse::<u64>()
        .unwrap();
    pub static ref INSCRIBE_FILTER_ENABLE: bool = std::env::var("INSCRIBE_FILTER_ENABLE")
        .expect("INSCRIBE_FILTER_ENABLE must be set")
        .parse::<bool>()
        .unwrap();
    pub static ref REINDEX: bool = std::env::var("REINDEX")
        .unwrap_or("false".to_string())
        .parse::<bool>()
        .unwrap();
    pub static ref START_BLOCK_MINT: u64 = std::env::var("START_BLOCK_MINT")
        .expect("START_BLOCK_MINT must be set")
        .parse::<u64>()
        .unwrap();
    pub static ref WORKER_BUFFER_LENGTH: usize = std::env::var("WORKER_BUFFER_LENGTH")
        .expect("WORKER_BUFFER_LENGTH must be set")
        .parse::<usize>()
        .unwrap();
    pub static ref DB_PATH: String = std::env::var("DB_PATH").unwrap_or(DEFAULT_DB_PATH.to_string());
    pub static ref TOKEN_PROTOCOL: String = std::env::var("TOKEN_PROTOCOL")
        .expect("TOKEN_PROTOCOL must be set")
        .parse::<String>()
        .unwrap();
    pub static ref HTTP_BIND: String = std::env::var("HTTP_BIND")
        .expect("HTTP_BIND must be set")
        .parse::<String>()
        .unwrap();
    pub static ref HTTP_PORT: u16 = std::env::var("HTTP_PORT")
        .expect("HTTP_PORT must be set")
        .parse::<u16>()
        .unwrap();
    pub static ref API_ONLY: bool = std::env::var("API_ONLY")
        .unwrap_or("false".to_string())
        .parse::<bool>()
        .unwrap();
    pub static ref OPEN_FILES_LIMIT: u64 = std::env::var("OPEN_FILES_LIMIT")
        .unwrap_or("0".to_string())
        .parse::<u64>()
        .unwrap();
    pub static ref MARKET_ADDRESS_LIST : Vec<String> = vec![
        "0xa66d17a09dc205b90e618c52fefc95d11bef6c91".to_string(),
        "0xa8ab79a4172713e2d77e31ad9594c72483299bfe".to_string(),
    ];
}
