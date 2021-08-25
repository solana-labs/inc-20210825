use {crate::config::Config, solana_sdk::pubkey::Pubkey};

pub fn run(_config: Config, owners: Vec<Pubkey>, mints: Vec<Pubkey>) {
    println!("audit");
    println!("owners:");
    println!("{:?}", owners);
    println!("mints:");
    println!("{:?}", mints);
}
