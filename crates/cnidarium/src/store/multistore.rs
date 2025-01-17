use std::{fmt::Display, sync::Arc};

use super::substore::SubstoreConfig;

/// A collection of substore, each with a unique prefix.
#[derive(Debug, Clone)]
pub struct MultistoreConfig {
    pub main_store: Arc<SubstoreConfig>,
    pub substores: Vec<Arc<SubstoreConfig>>,
}

impl MultistoreConfig {
    pub fn iter(&self) -> impl Iterator<Item = &Arc<SubstoreConfig>> {
        self.substores.iter()
    }

    /// Returns the substore matching the key's prefix, return `None` otherwise.
    pub fn find_substore(&self, key: &[u8]) -> Option<Arc<SubstoreConfig>> {
        if key.is_empty() {
            return Some(self.main_store.clone());
        }

        // Note: This is a linear search, but the number of substores is small.
        self.substores
            .iter()
            .find(|s| key.starts_with(s.prefix.as_bytes()))
            .cloned()
    }

    /// Route a key to a substore, and return the truncated key and the corresponding `SubstoreConfig`.
    ///
    /// This method is used for ordinary key-value operations.
    ///
    /// Note: since this method implements the routing logic for the multistore,
    /// callers might prefer [`MultistoreConfig::match_prefix_str`] if they don't
    /// need to route the key.
    ///
    /// # Routing
    /// + If the key is a total match for the prefix, the **main store** is returned.
    /// + If the key is not a total match for the prefix, the prefix is removed from  
    ///   the key and the key is routed to the substore matching the prefix.
    /// + If the key does not match any prefix, the key is routed to the **main store**.
    /// + If a delimiter is prefixing the key, it is removed.
    ///
    /// # Examples
    /// `prefix_a/key` -> `key` in `substore_a`
    /// `prefix_akey` -> `prefix_akey` in `main_store
    /// `prefix_a` -> `prefix_a` in `main_store`
    /// `prefix_a/` -> `prefix_a/` in `main_store
    /// `nonexistent_prefix` -> `nonexistent_prefix` in `main_store`
    pub fn route_key_str<'a>(&self, key: &'a str) -> (&'a str, Arc<SubstoreConfig>) {
        let config = self
            .find_substore(key.as_bytes())
            .unwrap_or_else(|| self.main_store.clone());

        // If the key is a total match, we want to return the key bound to the
        // main store. This is where the root hash of the prefix tree is located.
        if key == config.prefix {
            return (key, self.main_store.clone());
        }

        let truncated_key = key
            .strip_prefix(&config.prefix)
            .expect("key has the prefix of the matched substore");

        // If the key does not contain a delimiter, we return the original key
        // routed to the main store. This is because we do not want to allow
        // collisions e.g. `prefix_a/key` and `prefix_akey`.
        let Some(matching_key) = truncated_key.strip_prefix('/') else {
            return (key, self.main_store.clone());
        };

        // If the matching key is empty, we return the original key routed to
        // the main store. This is because we do not want to allow empty keys
        // in the substore.
        if matching_key.is_empty() {
            (key, self.main_store.clone())
        } else {
            (matching_key, config)
        }
    }

    /// Route a key to a substore, and return the truncated key and the corresponding `SubstoreConfig`.
    ///
    /// This method is used for ordinary key-value operations.
    ///
    /// Note: since this method implements the routing logic for the multistore,
    /// callers might prefer [`MultistoreConfig::match_prefix_bytes`] if they don't
    /// need to route the key.
    ///
    /// # Routing
    /// + If the key is a total match for the prefix, the **main store** is returned.
    /// + If the key is not a total match for the prefix, the prefix is removed from  
    ///   the key and the key is routed to the substore matching the prefix.
    /// + If the key does not match any prefix, the key is routed to the **main store**.
    /// + If a delimiter is prefixing the key, it is removed.
    ///
    /// # Examples
    /// `prefix_a/key` -> `key` in `substore_a`
    /// `prefix_a` -> `prefix_a` in `main_store`
    /// `prefix_a/` -> `prefix_a/` in `main_store`
    /// `nonexistent_prefix` -> `nonexistent_prefix` in `main_store`
    pub fn route_key_bytes<'a>(&self, key: &'a [u8]) -> (&'a [u8], Arc<SubstoreConfig>) {
        let config = self
            .find_substore(key)
            .unwrap_or_else(|| self.main_store.clone());

        // If the key is a total match for the prefix, we return the original key
        // routed to the main store. This is where subtree root hashes are stored.
        if key == config.prefix.as_bytes() {
            return (key, self.main_store.clone());
        }

        let truncated_key = key
            .strip_prefix(config.prefix.as_bytes())
            .expect("key has the prefix of the matched substore");

        // If the key does not contain a delimiter, we return the original key
        // routed to the main store. This is because we do not want to allow
        // collisions e.g. `prefix_a/key` and `prefix_akey`.
        let Some(matching_key) = truncated_key.strip_prefix(b"/") else {
            return (key, self.main_store.clone());
        };

        // If the matching key is empty, we return the original key routed to
        // the main store. This is because we do not want to allow empty keys
        // in the substore.
        if matching_key.is_empty() {
            (key, self.main_store.clone())
        } else {
            (matching_key, config)
        }
    }

    /// Returns the truncated prefix and the corresponding `SubstoreConfig`.
    ///
    /// This method is used to implement prefix iteration.
    ///
    /// Unlike [`MultistoreConfig::route_key_str`], this method does not do any routing.
    /// It simply finds the substore matching the prefix, strip the prefix and delimiter,
    /// and returns the truncated prefix and the corresponding `SubstoreConfig`.
    ///
    /// # Examples
    /// `prefix_a/key` -> `key` in `substore_a`
    /// `prefix_a` -> "" in `substore_a`
    /// `prefix_a/` -> "" in `substore_a`
    /// `nonexistent_prefix` -> "" in `main_store`
    pub fn match_prefix_str<'a>(&self, prefix: &'a str) -> (&'a str, Arc<SubstoreConfig>) {
        let config = self
            .find_substore(prefix.as_bytes())
            .unwrap_or_else(|| self.main_store.clone());

        let truncated_prefix = prefix
            .strip_prefix(&config.prefix)
            .expect("key has the prefix of the matched substore");

        let truncated_prefix = truncated_prefix
            .strip_prefix('/')
            .unwrap_or(truncated_prefix);
        (truncated_prefix, config)
    }

    /// Returns the truncated prefix and the corresponding `SubstoreConfig`.
    ///
    /// Unlike [`MultistoreConfig::route_key_str`], this method does not do any routing.
    /// It simply finds the substore matching the prefix, strip the prefix and delimiter,
    /// and returns the truncated prefix and the corresponding `SubstoreConfig`.
    ///
    /// This method is used to implement prefix iteration.
    ///
    /// # Examples
    /// `prefix_a/key` -> `key` in `substore_a`
    /// `prefix_a` -> "" in `substore_a`
    /// `prefix_a/` -> "" in `substore_a`
    /// `nonexistent_prefix` -> "" in `main_store`
    pub fn match_prefix_bytes<'a>(&self, prefix: &'a [u8]) -> (&'a [u8], Arc<SubstoreConfig>) {
        let config = self
            .find_substore(prefix)
            .unwrap_or_else(|| self.main_store.clone());

        let truncated_prefix = prefix
            .strip_prefix(config.prefix.as_bytes())
            .expect("key has the prefix of the matched substore");

        let truncated_prefix = truncated_prefix
            .strip_prefix(b"/")
            .unwrap_or(truncated_prefix);
        (truncated_prefix, config)
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

/// Tracks the latest version of each substore, and wraps a `MultistoreConfig`.
#[derive(Default, Debug)]
pub struct MultistoreCache {
    pub config: MultistoreConfig,
    pub substores: std::collections::BTreeMap<Arc<SubstoreConfig>, jmt::Version>,
}

impl Display for MultistoreCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for (substore, version) in &self.substores {
            s.push_str(&format!("{}: {}\n", substore.prefix, version));
        }
        write!(f, "{}", s)
    }
}

impl MultistoreCache {
    pub fn from_config(config: MultistoreConfig) -> Self {
        Self {
            config,
            substores: std::collections::BTreeMap::new(),
        }
    }

    pub fn set_version(&mut self, substore: Arc<SubstoreConfig>, version: jmt::Version) {
        self.substores.insert(substore, version);
    }

    pub fn get_version(&self, substore: &Arc<SubstoreConfig>) -> Option<jmt::Version> {
        self.substores.get(substore).cloned()
    }
}
