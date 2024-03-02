use crate::args::Args;
use clap::Parser;

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
    pub static ref TICK_MAX_LEN: usize = ARGS.tick_max_len;
    pub static ref WORKER_COUNT: u64 = ARGS.worker_count;
    pub static ref CONFIRM_BLOCK: u64 = ARGS.confirm_block;
    pub static ref CHAIN_ID: u64 = ARGS.chain_id;
    pub static ref WEB3_PROVIDER: String = ARGS.web3_provider.clone();
    pub static ref START_BLOCK: u64 = ARGS.start_block;
    pub static ref REINDEX: bool = ARGS.reindex;
    pub static ref START_BLOCK_MINT: u64 = ARGS.start_block_mint;
    pub static ref WORKER_BUFFER_LENGTH: usize = ARGS.worker_buffer_length;
    pub static ref DB_PATH: String = ARGS.db_path.clone();
    pub static ref TOKEN_PROTOCOL: String = ARGS.token_protocol.clone();
    pub static ref HTTP_BIND: String = ARGS.http_bind.clone();
    pub static ref HTTP_PORT: u16 = ARGS.http_port;
    pub static ref API_ONLY: bool = ARGS.api_only;
    pub static ref OPEN_FILES_LIMIT: u64 = ARGS.open_files_limit;
    pub static ref MARKET_ADDRESS_LIST: Vec<String> = ARGS.market_address_list.clone();
}
