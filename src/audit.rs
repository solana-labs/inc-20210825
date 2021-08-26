use {
    crate::config::Config,
    serde::{Deserialize, Serialize},
    solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config,
    solana_sdk::{pubkey::Pubkey, signature::Signature, signer::Signer},
    solana_transaction_status::{
        EncodedTransaction, EncodedTransactionWithStatusMeta, UiInstruction, UiMessage,
        UiParsedInstruction, UiTransactionEncoding,
    },
    std::{collections::HashMap, str::FromStr},
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct DelegateTransfer {
    pub transaction_id: Signature,
    pub signer: Pubkey,
    pub amount: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct OwnerChange {
    pub transaction_id: Signature,
    pub signer: Pubkey,
    pub new_owner: Pubkey,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct DelegateChange {
    pub transaction_id: Signature,
    pub signer: Pubkey,
    pub new_delegate: Option<Pubkey>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct TokenAccountEntry {
    token_account_address: Pubkey,
    mint_address: Pubkey,
    delegate_address: Option<Pubkey>,
    delegate_transfers: Vec<DelegateTransfer>,
    owner_changes: Vec<OwnerChange>,
    delegate_changes: Vec<DelegateChange>,
}

impl TokenAccountEntry {
    pub fn new(token_account_address: Pubkey, mint_address: Pubkey) -> Self {
        Self {
            token_account_address,
            mint_address,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Report {
    wallets: HashMap<Pubkey, TokenAccountEntry>,
}

pub fn run(config: Config, owners: Vec<Box<dyn Signer>>, mints: Vec<Pubkey>) {
    println!("audit");
    let mut report = Report {
        wallets: HashMap::new(),
    };
    const SIGNATURES_LIMIT: usize = 1000;
    crate::for_all_spl_token_accounts(
        &config,
        owners.as_slice(),
        mints.as_slice(),
        |config, owner, address, account| {
            let rpc_client = &config.rpc_client;
            let owner_pubkey = owner.pubkey();
            let wallet_entry = report
                .wallets
                .entry(owner_pubkey)
                .or_insert_with(|| TokenAccountEntry::new(*address, account.mint));
            let mut before = Option::<Signature>::None;
            loop {
                let request_config = GetConfirmedSignaturesForAddress2Config {
                    before,
                    limit: Some(SIGNATURES_LIMIT),
                    ..GetConfirmedSignaturesForAddress2Config::default()
                };
                #[allow(deprecated)]
                let sigs = rpc_client
                    .get_confirmed_signatures_for_address2_with_config(&address, request_config)
                    .unwrap();

                before = if sigs.len() < SIGNATURES_LIMIT {
                    None
                } else {
                    sigs.last()
                        .and_then(|s| Signature::from_str(s.signature.as_str()).ok())
                };

                let sigs = sigs.iter().filter_map(|sig_with_status| {
                    if sig_with_status.err.is_some() {
                        None
                    } else {
                        Signature::from_str(sig_with_status.signature.as_str()).ok()
                    }
                });

                for sig in sigs {
                    #[allow(deprecated)]
                    let transaction = rpc_client
                        .get_confirmed_transaction(&sig, UiTransactionEncoding::JsonParsed)
                        .unwrap()
                        .transaction;
                    let EncodedTransactionWithStatusMeta { transaction, meta } = transaction;
                    let inner_ix = meta.and_then(|meta| {
                        meta.inner_instructions
                            .map(|ixs| ixs.into_iter().map(|ixs| ixs.instructions).flatten())
                    });
                    let mut instructions =
                        if let EncodedTransaction::Json(transaction) = transaction {
                            if let UiMessage::Parsed(message) = transaction.message {
                                message.instructions
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        };

                    if let Some(inner_ix) = inner_ix {
                        instructions.extend(inner_ix);
                    }

                    // spl token instructions will be parsed
                    let instructions = instructions
                        .into_iter()
                        .filter_map(|ix| {
                            if let UiInstruction::Parsed(UiParsedInstruction::Parsed(instruction)) =
                                ix
                            {
                                let program_id =
                                    Pubkey::from_str(instruction.program_id.as_str()).unwrap();
                                if program_id == spl_token::id() {
                                    Some((program_id, instruction.parsed))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .for_each(|(pid, ix)| println!("{:?}", ix));
                }

                // last
                if before.is_none() {
                    break;
                }
            }
        },
    )
    .unwrap();
}
