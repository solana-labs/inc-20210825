use {
    crate::config::Config,
    solana_sdk::{
        message::Message, program_option::COption, pubkey::Pubkey, signature::Signer,
        transaction::Transaction,
    },
    spl_token::{self, instruction::revoke, state::Account},
};

fn cleanup(config: &Config, owner: &dyn Signer, address: &Pubkey, account: &Account) {
    if let COption::Some(delegate) = account.delegate {
        println!("revoking delegate {} for account {}", delegate, address);
        let rpc_client = &config.rpc_client;
        let revoke_ix = revoke(&spl_token::id(), &address, &owner.pubkey(), &[]).unwrap();
        let fee_payer = config.fee_payer.pubkey();
        let message = Message::new(&[revoke_ix], Some(&fee_payer));
        let (blockhash, fee_calculator) = rpc_client.get_recent_blockhash().unwrap();
        let fee_payer_balance = rpc_client.get_balance(&fee_payer).unwrap();
        let fee = fee_calculator.calculate_fee(&message);
        if fee_payer_balance < fee {
            eprintln!("fee payer ({}) insufficient funds!", fee_payer);
            std::process::exit(1);
        }
        let mut transaction = Transaction::new_unsigned(message);
        transaction.sign(&[owner, config.fee_payer.as_ref()], blockhash);

        let txid = rpc_client
            .send_and_confirm_transaction_with_spinner(&transaction)
            .unwrap();
        // TODO: print CLI command on failure?
        println!("txid: {}", txid);
    }
}

pub fn run(config: Config, owners: Vec<Box<dyn Signer>>, mints: Vec<Pubkey>) {
    println!("cleanup");
    crate::for_all_spl_token_accounts(&config, owners.as_slice(), mints.as_slice(), cleanup)
        .unwrap();
}
