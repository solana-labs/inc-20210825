use {solana_client::rpc_client::RpcClient, solana_sdk::signature::Signer};

pub struct Config {
    pub rpc_client: RpcClient,
    pub json_rpc_url: String,
    pub fee_payer: Box<dyn Signer>,
    pub dry_run: bool,
    pub verbose: bool,
}
