use std::collections::HashMap;

use penumbra_sdk_asset::asset::Metadata;

use crate::config::Symbol;

/// Contains information about the assets we know.
///
/// This allows human friendly representations of prices, names, to map
/// to the machine-equivalent data.
pub struct Registry {
    by_symbol: HashMap<String, Metadata>,
}

impl Registry {
    /// Create a Registry from an iterator over metadata.
    pub fn from_metadata(metadata: &mut dyn Iterator<Item = &Metadata>) -> Self {
        let by_symbol = metadata
            .map(|m| (m.symbol().to_string(), m.clone()))
            .collect();
        Self { by_symbol }
    }

    /// Lookup metadata given a symbol.
    pub fn lookup(&self, symbol: &Symbol) -> Option<&Metadata> {
        self.by_symbol.get(symbol.as_ref())
    }
}
