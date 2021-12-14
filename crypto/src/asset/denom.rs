use std::{collections::BTreeMap, fmt::Display, sync::Arc};

use ark_ff::fields::PrimeField;

use crate::{asset, Fq};

pub struct BaseDenom {
    inner: Arc<Inner>,
}

pub struct DisplayDenom {
    inner: Arc<Inner>,
    // Indexes into the `units` field on `Inner`.
    // The units field is always sorted by priority order.
    unit_index: u8,
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
        (0..self.inner.units.len())
            .map(|unit_index| DisplayDenom {
                unit_index: unit_index as u8,
                inner: self.inner.clone(),
            })
            .collect()
    }

    pub fn default_unit(&self) -> DisplayDenom {
        self.units()
            .get(0)
            .expect("there must be at least one unit");

        DisplayDenom {
            unit_index: 0,
            inner: self.inner.clone(),
        }
    }
}

impl DisplayDenom {
    pub fn base(&self) -> BaseDenom {
        BaseDenom {
            inner: self.inner.clone(),
        }
    }

    pub fn exponent(&self) -> u8 {
        self.inner
            .units
            .get(self.unit_index as usize)
            .expect("there must be an entry for unit_index")
            .exponent
    }

    pub fn raw_name(&self) -> String {
        self.inner
            .units
            .get(self.unit_index as usize)
            .expect("there must be an entry for unit_index")
            .denom
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
