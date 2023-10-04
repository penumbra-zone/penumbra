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
    pub fn find_substore(&self, key: &str) -> Option<Arc<SubstoreConfig>> {
        // Note: This is a linear search, but the number of substores is small.
        self.substores
            .iter()
            // .skip(1) /* skip the transparent substore - not necessary if we split them in the struct */
            .find(|s| key.starts_with(&s.prefix))
            .cloned()
    }

    /// Route the key to the correct substore, or the transparent store if no prefix matches.
    /// Returns the truncated key, and the target snapshot.
    pub fn route_key<'a>(&self, key: &'a str) -> (&'a str, Arc<SubstoreConfig>) {
        match self.find_substore(key) {
            Some(config) => (
                key.strip_prefix(&config.prefix)
                    .expect("key has the prefix of the matched substore"),
                config,
            ),
            None => (key, self.transparent_store.clone()),
        }
    }
}
