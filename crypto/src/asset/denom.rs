use std::{collections::BTreeMap, sync::Arc};

use ark_ff::fields::PrimeField;

use crate::{asset, Fq};

pub struct BaseDenom {
    inner: Arc<Inner>,
}

pub struct DisplayDenom {
    inner: Arc<Inner>,
    // todo: specify which display denom?
}

// These are constructed by the asset registry.
pub(super) struct Inner {
    id: asset::Id,
    /// Sorted by priority order.
    units: Vec<Unit>,
}

struct Unit {
    pub exponent: u8,
    pub denom: String,
}

impl Inner {
    // TODO: params?
    pub fn new(raw_str: &str, units: Vec<Unit>) -> Self {
        let id = asset::Id(Fq::from_le_bytes_mod_order(
            // XXX choice of hash function?
            blake2b_simd::Params::default()
                .personal(b"Penumbra_AssetID")
                .hash(raw_str.as_bytes())
                .as_bytes(),
        ));

        Self { id, units }
    }
}

impl BaseDenom {
    pub fn id(&self) -> asset::Id {
        self.inner.id.clone()
    }

    /// Return a list of display units for this denomination, in priority order.
    ///
    /// There will always be at least one display denomination.
    pub fn units(&self) -> Vec<DisplayDenom> {
        todo!()
    }

    pub fn default_unit(&self) -> DisplayDenom {
        todo!()
    }
}

impl DisplayDenom {
    pub fn base(&self) -> BaseDenom {
        BaseDenom {
            inner: self.inner.clone(),
        }
    }

    // TODO: API for actually working with displayed denominations?
    // - format-style printing of amounts of base denom in displaydenom units
    // - conversion of numbers? (tricky? do you use floats? or what fractions?)
}

impl From<BaseDenom> for asset::Id {
    fn from(base: BaseDenom) -> asset::Id {
        base.id()
    }
}
