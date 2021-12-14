use once_cell::sync::Lazy;

use crate::asset::{denom, BaseDenom, DisplayDenom};

pub static REGISTRY: Lazy<Registry> = Lazy::new(|| {
    // specify regexes here?
    todo!()
});

pub struct Registry {
    //
}

impl Registry {
    pub fn parse_base(&self, raw_denom: &str) -> Option<BaseDenom> {
        todo!()
    }

    pub fn parse_display(&self, raw_denom: &str) -> Option<DisplayDenom> {
        todo!()
    }
}
