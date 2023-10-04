use std::sync::Arc;

use super::substore::SubstoreConfig;

/// A collection of substore, each with a unique prefix.
#[derive(Debug, Clone)]
pub struct Multistore(Vec<Arc<SubstoreConfig>>);

impl Multistore {
    pub fn new(substores: Vec<Arc<SubstoreConfig>>) -> Self {
        Self(substores)
    }

    pub fn get(&self, prefix: &str) -> Option<Arc<SubstoreConfig>> {
        self.0.iter().find(|s| s.prefix == prefix).cloned()
    }

    /// Returns the substore with the corresponding prefix, `None` otherwise.
    pub fn find_substore(&self, key: &str) -> Option<Arc<SubstoreConfig>> {
        // Note: This is a linear search, but the number of substores is small.
        self.0.iter().find(|s| key.starts_with(&s.prefix)).cloned()
    }
}
