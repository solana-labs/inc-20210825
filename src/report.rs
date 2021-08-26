use {
    crate::token::TokenAccountEntry,
    serde::{Deserialize, Serialize},
    solana_sdk::pubkey::Pubkey,
    std::collections::HashMap,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Report {
    pub entries_by_token_address: HashMap<Pubkey, TokenAccountEntry>,
}
