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
    pub signer: String, // was Pubkey
    pub amount: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct DelegateBurn {
    pub transaction_id: Signature,
    pub signer: Pubkey,
    pub amount: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct OwnerChange {
    pub transaction_id: Signature,
    pub signer: String,    // was Pubkey
    pub new_owner: String, // was Pubkey
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct DelegateChange {
    pub transaction_id: Signature,
    pub signer: String,       // was Pubkey
    pub new_delegate: String, // was Option<Pubkey>
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct TokenAccountEntry {
    owner: Pubkey,
    mint: Pubkey,
    // TODO: intra slot tx order can't be guranteed.... so there is no perfect way to precisely track the latest delegate_address
    //delegate_address: Option<Pubkey>
    delegate_transfers: Vec<DelegateTransfer>,
    owner_changes: Vec<OwnerChange>,
    delegate_changes: Vec<DelegateChange>,
}

impl TokenAccountEntry {
    pub fn new(owner: Pubkey, mint: Pubkey) -> Self {
        Self {
            owner,
            mint,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Report {
    wallets: HashMap<Pubkey, TokenAccountEntry>, // key = token address
}

fn scan_ix(token_account: &mut TokenAccountEntry, sig: Signature, ix: &serde_json::Value) -> bool {
    const CONSUMED: bool = false;
    const IGNORED: bool = false;

    match ix.get("type").as_ref() {
        Some(serde_json::value::Value::String(a)) => {
            let ix_info = ix.get("info");
            match a.as_ref() {
                "transfer" | "transferChecked" => {
                    token_account.delegate_transfers.push(DelegateTransfer {
                        transaction_id: sig,
                        signer: format!("{}", ix_info.unwrap().get("authority").unwrap()),
                        // TODO: todo: properly handle this field!
                        amount: format!(
                            "{}",
                            ix_info
                                .unwrap()
                                .get("tokenAmount")
                                .map(|ta| ta.get("uiAmountString").unwrap())
                                .unwrap_or_else(|| ix_info.unwrap().get("amount").unwrap())
                        ),
                    });
                    CONSUMED
                }
                "approve" | "approveChecked" => {
                    token_account.delegate_changes.push(DelegateChange {
                        transaction_id: sig,
                        signer: format!("{}", ix_info.unwrap().get("owner").unwrap()),
                        new_delegate: format!("{}", ix_info.unwrap().get("delegate").unwrap()),
                    });
                    CONSUMED
                }
                "setAuthority" => {
                    match (ix_info.map(|info| info.get("authorityType").unwrap())).as_ref() {
                        Some(serde_json::value::Value::String(a)) => match a.as_ref() {
                            "accountOwner" => {
                                token_account.owner_changes.push(OwnerChange {
                                    transaction_id: sig,
                                    new_owner: format!(
                                        "{}",
                                        ix_info.unwrap().get("newAuthority").unwrap()
                                    ),
                                    signer: format!(
                                        "{}",
                                        ix_info.unwrap().get("authority").unwrap()
                                    ),
                                });
                                CONSUMED
                            }
                            _ => !CONSUMED,
                        },
                        _ => !CONSUMED,
                    }
                }
                "initializeAccount" => IGNORED,
                "mintTo" | "mintToChecked" => IGNORED,
                _ => !CONSUMED,
            }
        }
        _ => !CONSUMED,
    }
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
            let mut token_account_entry = report
                .wallets
                //.entry((owner_pubkey, account.mint))
                .entry(*address)
                .or_insert_with(|| TokenAccountEntry::new(owner_pubkey, account.mint));
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

                // Exclude any transactions which failed
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

                    // only spl token instructions will be parsed
                    instructions
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
                        // program_id must be the tokenkeg according the previous .filter_map()
                        .filter(|(_program_id, ix)| scan_ix(&mut token_account_entry, sig, ix))
                        .for_each(|(program_id, ix)| {
                            dbg!(("unknown instruction!", program_id, ix));
                        });
                }

                // last
                if before.is_none() {
                    break;
                }
            }
        },
    )
    .unwrap();

    // nicely format! or csv?
    dbg!(report);
}
