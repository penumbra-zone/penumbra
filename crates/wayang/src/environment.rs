use crate::config::{Config, SymbolPair};

pub struct Environment {
    pair: SymbolPair,
}

impl Environment {
    pub fn pair(&self) -> &SymbolPair {
        &self.pair
    }
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            pair: config.pair.clone(),
        })
    }
}
