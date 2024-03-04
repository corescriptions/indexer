use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The maximum length of tick
    #[arg(long, env, default_value = "32")]
    pub tick_max_len: usize,

    /// The number of workers for sync blocks data
    #[arg(long, env, default_value = "1")]
    pub worker_count: u64,

    /// The number of confirm block, when inscribe a new block data
    #[arg(long, env, default_value = "1")]
    pub confirm_block: u64,

    /// The chain id of the network
    #[arg(long, env, default_value = "1")]
    pub chain_id: u64,

    /// The web3 provider url
    #[arg(long, env)]
    pub web3_provider: String,

    /// The start block number for sync and inscribe
    #[arg(long, env)]
    pub start_block: u64,

    /// The start block number for sync and token mint
    #[arg(long, env)]
    pub start_block_mint: u64,

    /// Reindex the block data
    #[arg(long, env, default_value = "false")]
    pub reindex: bool,

    /// The length of worker sync buffer
    #[arg(long, env, default_value = "64")]
    pub worker_buffer_length: usize,

    /// The path of database
    #[arg(long, env, default_value = "./data")]
    pub db_path: String,

    /// The token protocol
    #[arg(long, env, default_value = "erc-20")]
    pub token_protocol: String,

    /// The rpc http bind address
    #[arg(long, env, default_value = "127.0.0.1")]
    pub http_bind: String,

    /// The rpc http port
    #[arg(long, env, default_value = "8711")]
    pub http_port: u16,

    /// Run in api only mode
    #[arg(long, env, default_value = "false")]
    pub api_only: bool,

    /// The open files limit
    #[arg(long, env, default_value = "10240")]
    pub open_files_limit: u64,

    /// The market address list
    #[arg(long, default_value = "0xa66d17a09dc205b90e618c52fefc95d11bef6c91,0xa8ab79a4172713e2d77e31ad9594c72483299bfe")]
    pub market_address_list: String,
}

pub fn parse() -> Args {
    dotenv::dotenv().ok();
    let args = Args::parse();
    args
}
