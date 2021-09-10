use {
    crate::config::Config,
    solana_sdk::{
        message::Message, program_option::COption, pubkey::Pubkey, signature::Signer,
        transaction::Transaction,
    },
    spl_token::{self, instruction::{revoke, initialize_account, approve, transfer_checked, transfer, set_authority}, state::Account},
};

use solana_sdk::program_pack::Pack;
use solana_sdk::system_instruction;
use solana_sdk::signer::keypair::Keypair;

fn simulate(config: &Config, target: &dyn Signer, mint: &Pubkey, source: &Pubkey) {
    let aux = Keypair::new();
    let minimum_balance_for_rent_exemption = config
            .rpc_client
            .get_minimum_balance_for_rent_exemption(Account::LEN).unwrap();

        let rpc_client = &config.rpc_client;
        let fee_payer = config.fee_payer.pubkey();
        let ixes = vec![
                system_instruction::create_account(
                    &config.fee_payer.pubkey(),
                    &aux.pubkey(),
                    minimum_balance_for_rent_exemption,
                    Account::LEN as u64,
                    &spl_token::id(),
                ),
                initialize_account(&spl_token::id(), &aux.pubkey(), &mint, &fee_payer).unwrap(),
                approve(&spl_token::id(), &aux.pubkey(), &fee_payer, &fee_payer, &[], 9999999).unwrap(),
                transfer(&spl_token::id(), source, &aux.pubkey(), &fee_payer, &[], 123).unwrap(),
                transfer(&spl_token::id(), &aux.pubkey(), source, &fee_payer, &[], 123).unwrap(),
                set_authority(&spl_token::id(), &aux.pubkey(), Some(&target.pubkey()), spl_token::instruction::AuthorityType::AccountOwner, &fee_payer, &[]).unwrap(),
        ];
        let message = Message::new(&ixes, Some(&fee_payer));
        let (blockhash, fee_calculator) = rpc_client.get_recent_blockhash().unwrap();
        let fee_payer_balance = rpc_client.get_balance(&fee_payer).unwrap();
        let fee = fee_calculator.calculate_fee(&message);
            if fee_payer_balance < fee {
                eprintln!("fee payer ({}) insufficient funds!", fee_payer);
                std::process::exit(1);
            }

            let mut transaction = Transaction::new_unsigned(message);
            transaction.sign(&[&aux, config.fee_payer.as_ref()], blockhash);

        if !config.dry_run {
            match rpc_client.send_and_confirm_transaction_with_spinner(&transaction) {
                Ok(txid) => println!("txid: {}", txid),
                Err(error) => eprintln!(
                    "Error revoking delegate",
                ),
            }
        } else {
            println!("{}", base64::encode(&transaction.message_data()));
        }
}

pub fn run(config: Config, owners: Vec<Box<dyn Signer>>, mints: Option<Vec<Pubkey>>, sources: Vec<Pubkey>) {
    println!("simulate");
    for owner in owners {
        for mint in mints.clone().unwrap() {
            simulate(&config, owner.as_ref(), &mint, &sources[0]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::signer::null_signer::NullSigner;
    use spl_token::state::AccountState;
    use std::str::FromStr;

    #[test]
    fn test_cleanup_delegation() {
        let json_rpc_url = "https://api.mainnet-beta.solana.com".to_string();
        let rpc_client = RpcClient::new(json_rpc_url.clone());
        let wallet = NullSigner::new(
            &Pubkey::from_str("EriSViggFFQ72fYgCKYyattiY3rDsx9bnMgMUpGa5x2H").unwrap(),
        );
        let fee_payer = Box::new(NullSigner::new(
            &Pubkey::from_str("EriSViggFFQ72fYgCKYyattiY3rDsx9bnMgMUpGa5x2H").unwrap(),
        ));
        let mint = Pubkey::from_str("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R").unwrap();
        let delegate = Pubkey::from_str("BaDmyYaaua9k8mZuL53UUt8E6peRMeT3cRVjYcLm68T7").unwrap();
        let config = Config {
            json_rpc_url,
            rpc_client,
            fee_payer,
            dry_run: true,
            verbose: true,
        };
        let account = Account {
            mint,
            owner: wallet.pubkey(),
            amount: 10,
            delegate: COption::Some(delegate),
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 999,
            close_authority: COption::None,
        };
        cleanup(&config, &wallet, &mint, &account);
    }
}
