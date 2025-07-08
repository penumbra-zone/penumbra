use penumbra_sdk_asset::asset::Metadata;
use penumbra_sdk_dex::DirectedTradingPair;
use std::collections::HashMap;

use super::{Symbol, SymbolPair};

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

    /// Try and convert a symbol pair to an actual trading pair.
    ///
    /// This will fail if either symbol isn't present in the registry, for any reason.
    pub fn convert_pair(&self, pair: &SymbolPair) -> Option<DirectedTradingPair> {
        let base_meta = self.lookup(&pair.base)?;
        let quote_meta = self.lookup(&pair.quote)?;
        Some(DirectedTradingPair::new(base_meta.id(), quote_meta.id()))
    }
}
