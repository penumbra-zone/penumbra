use std::sync::Arc;

use super::substore::SubstoreConfig;

/// A collection of substore, each with a unique prefix.
#[derive(Debug, Clone)]
pub struct Multistore {
    transparent_store: Arc<SubstoreConfig>,
    substores: Vec<Arc<SubstoreConfig>>,
}

impl Multistore {
    pub fn new(mut substores: Vec<Arc<SubstoreConfig>>) -> Self {
        assert!(
            !substores.is_empty(),
            "multistore must have at least one substore"
        );

        Self {
            transparent_store: substores.swap_remove(0),
            substores,
        }
    }

    pub fn _get(&self, prefix: &str) -> Option<Arc<SubstoreConfig>> {
        if prefix.is_empty() {
            Some(self.transparent_store.clone())
        } else {
            self.substores.iter().find(|s| s.prefix == prefix).cloned()
        }
    }

    /// Returns the substore matching the key's prefix, return `None` otherwise.
    pub fn find_substore(&self, key: &[u8]) -> Option<Arc<SubstoreConfig>> {
        let key = key.as_ref();
        // Note: This is a linear search, but the number of substores is small.
        self.substores
            .iter()
            .find(|s| key.starts_with(&s.prefix.as_bytes()))
            .cloned()
    }

    /// Route the key to the correct substore, or the transparent store if no prefix matches.
    /// Returns the truncated key, and the target snapshot.
    /// TODO: refactor this later. or not. it's repetitive but simple, and mean we don't have to do an expensive utf8 conversion
    pub fn route_key_str<'a>(&self, key: &'a str) -> (&'a str, Arc<SubstoreConfig>) {
        match self.find_substore(key.as_bytes()) {
            Some(config) => (
                key.strip_prefix(&config.prefix)
                    .expect("key has the prefix of the matched substore"),
                config,
            ),
            None => (key, self.transparent_store.clone()),
        }
    }

    /// Route the key to the correct substore, or the transparent store if no prefix matches.
    /// Returns the truncated key, and the target snapshot.
    pub fn route_key<'a>(&self, key: &'a [u8]) -> (&'a [u8], Arc<SubstoreConfig>) {
        match self.find_substore(key) {
            Some(config) => (
                key.strip_prefix(config.prefix.as_bytes())
                    .expect("key has the prefix of the matched substore"),
                config,
            ),
            None => (key, self.transparent_store.clone()),
        }
    }
}
