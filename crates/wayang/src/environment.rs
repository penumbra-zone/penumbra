use crate::options::{Options, SymbolPair};

pub struct Environment {
    pair: SymbolPair,
}

impl Environment {
    pub fn pair(&self) -> &SymbolPair {
        &self.pair
    }
    pub fn new(options: Options) -> Self {
        Self { pair: options.pair }
    }
}
