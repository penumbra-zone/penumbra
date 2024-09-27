use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use penumbra_keys::FullViewingKey;
use penumbra_num::Amount;

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct FvkEntry {
    pub fvk: FullViewingKey,
    pub wallet_id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct AccountConfig {
    original: FvkEntry,
    genesis_balance: Amount,
    // If the account was migrated, we update the entry here.
    migrations: Vec<FvkEntry>,
}

impl AccountConfig {
    pub fn new(original: FvkEntry, genesis_balance: Amount) -> Self {
        Self {
            original,
            genesis_balance,
            migrations: vec![],
        }
    }

    /// Get original/genesis FVK.
    pub fn original_fvk(&self) -> FullViewingKey {
        self.original.fvk.clone()
    }

    /// Get genesis balance.
    pub fn genesis_balance(&self) -> Amount {
        self.genesis_balance
    }

    /// Add migration to the account config.
    pub fn add_migration(&mut self, fvk_entry: FvkEntry) {
        self.migrations.push(fvk_entry);
    }

    /// Get the active wallet, which is the last migration or the original FVK if no migrations have occurred.
    pub fn active_wallet(&self) -> FvkEntry {
        if self.migrations.is_empty() {
            self.original.clone()
        } else {
            self.migrations
                .last()
                .expect("migrations must not be empty")
                .clone()
        }
    }

    pub fn active_fvk(&self) -> FullViewingKey {
        self.active_wallet().fvk
    }

    pub fn active_uuid(&self) -> Uuid {
        self.active_wallet().wallet_id
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PmonitorConfig {
    grpc_url: Url,
    accounts: Vec<AccountConfig>,
}

impl PmonitorConfig {
    pub fn new(grpc_url: Url, accounts: Vec<AccountConfig>) -> Self {
        Self { grpc_url, accounts }
    }

    pub fn grpc_url(&self) -> Url {
        self.grpc_url.clone()
    }

    pub fn accounts(&self) -> &Vec<AccountConfig> {
        &self.accounts
    }

    pub fn set_account(&mut self, index: usize, account: AccountConfig) {
        self.accounts[index] = account;
    }
}

/// Get the destination FVK from a migration memo.
pub fn parse_dest_fvk_from_memo(memo: &str) -> Result<FullViewingKey> {
    let re = Regex::new(r"Migrating balance from .+ to (.+)")?;
    if let Some(captures) = re.captures(memo) {
        if let Some(dest_fvk_str) = captures.get(1) {
            return dest_fvk_str
                .as_str()
                .parse::<FullViewingKey>()
                .map_err(|_| anyhow::anyhow!("Invalid destination FVK in memo"));
        }
    }
    Err(anyhow::anyhow!("Could not parse destination FVK from memo"))
}
