use {
    solana_account_decoder::UiAccountEncoding,
    solana_client::{
        client_error::Result as ClientResult,
        rpc_client::RpcClient,
        rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
        rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    },
    solana_sdk::{program_pack::Pack, pubkey::Pubkey, signer::Signer},
};

pub mod audit;
pub mod cleanup;
pub mod config;

pub fn for_all_spl_token_accounts<F>(
    rpc_client: &RpcClient,
    wallets: &[&dyn Signer],
    mints: &[Pubkey],
    f: F,
) -> ClientResult<()>
where
    F: FnMut((&&dyn Signer, Pubkey, spl_token::state::Account)) + Copy,
{
    for wallet in wallets {
        let filters = Some(vec![
            RpcFilterType::DataSize(spl_token::state::Account::LEN as u64),
            RpcFilterType::Memcmp(Memcmp {
                offset: 32,
                bytes: MemcmpEncodedBytes::Binary(bs58::encode(wallet.pubkey()).into_string()),
                encoding: None,
            }),
        ]);
        let account_config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..RpcAccountInfoConfig::default()
        };
        let config = RpcProgramAccountsConfig {
            filters,
            account_config,
            ..RpcProgramAccountsConfig::default()
        };

        rpc_client
            .get_program_accounts_with_config(&spl_token::id(), config)?
            .into_iter()
            .filter_map(|(addr, acct)| {
                let token_account = spl_token::state::Account::unpack(&acct.data).ok();
                if token_account.is_none() {
                    eprintln!("unexpected account data at {}:", addr);
                }
                Some(addr).zip(token_account)
            })
            .filter(|(_address, account)| mints.contains(&account.mint))
            .map(|(address, account)| (wallet, address, account))
            .for_each(f);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signer::null_signer::NullSigner;
    use std::str::FromStr;

    #[test]
    fn test_for_all_spl_token_accounts() {
        let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
        let wallet = NullSigner::new(
            &Pubkey::from_str("EriSViggFFQ72fYgCKYyattiY3rDsx9bnMgMUpGa5x2H").unwrap(),
        );
        let mint = Pubkey::from_str("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R").unwrap();
        for_all_spl_token_accounts(
            &rpc_client,
            &vec![&wallet],
            &[mint],
            |(wallet, address, account)| {
                println!(
                    "owner: {}\naddress: {}\naccount: {:?}",
                    wallet.pubkey(),
                    address,
                    account
                );
            },
        )
        .unwrap();
    }
}
