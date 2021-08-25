use {
    crate::config::Config,
    solana_sdk::{pubkey::Pubkey, signature::Signer},
};

pub fn run(_config: Config, owners: Vec<(Box<dyn Signer>, Pubkey)>, mints: Vec<Pubkey>) {
    println!("cleanup");
    println!("owners:");
    println!("{:?}", owners);
    println!("mints:");
    println!("{:?}", mints);
}
