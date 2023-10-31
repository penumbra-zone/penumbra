use std::sync::Arc;

use super::substore::SubstoreConfig;

/// A collection of substore, each with a unique prefix.
#[derive(Debug, Clone)]
pub struct MultistoreConfig {
    pub main_store: Arc<SubstoreConfig>,
    pub substores: Vec<Arc<SubstoreConfig>>,
}

impl MultistoreConfig {
    /// Returns the substore matching the key's prefix, return `None` otherwise.
    pub fn find_substore(&self, key: &[u8]) -> Arc<SubstoreConfig> {
        let key = key.as_ref();
        // Note: This is a linear search, but the number of substores is small.
        self.substores
            .iter()
            .find(|s| key.starts_with(&s.prefix.as_bytes()))
            .cloned()
            .unwrap_or(self.main_store.clone())
    }

    /// Route the key to the correct substore, or the transparent store if no prefix matches.
    /// Returns the truncated key, and the target snapshot.
    pub fn route_key_str<'a>(&self, key: &'a str) -> (&'a str, Arc<SubstoreConfig>) {
        let config = self.find_substore(key.as_bytes());
        let key = key
            .strip_prefix(&config.prefix)
            .expect("key has the prefix of the matched substore");
        (key, config)
    }

    /// Route the key to the correct substore, or the transparent store if no prefix matches.
    /// Returns the truncated key, and the target snapshot.
    pub fn route_key_bytes<'a>(&self, key: &'a [u8]) -> (&'a [u8], Arc<SubstoreConfig>) {
        let config = self.find_substore(key);
        let key = key
            .strip_prefix(config.prefix.as_bytes())
            .expect("key has the prefix of the matched substore");
        (key, config)
    }
}

impl Default for MultistoreConfig {
    fn default() -> Self {
        Self {
            main_store: Arc::new(SubstoreConfig::new("")),
            substores: vec![],
        }
    }
}
