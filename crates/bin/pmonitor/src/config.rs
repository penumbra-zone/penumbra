use serde::{Deserialize, Serialize};
use url::Url;

use penumbra_keys::FullViewingKey;

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct FvkEntry {
    pub fvk: FullViewingKey,
    pub path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct AccountConfig {
    pub original: FvkEntry,
    // If the account was migrated, we update the entry here.
    pub migrations: Vec<FvkEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PmonitorConfig {
    pub grpc_url: Url,
    pub accounts: Vec<AccountConfig>,
}
