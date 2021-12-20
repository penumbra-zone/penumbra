use std::{
    cmp::{Eq, PartialEq},
    fmt::{Debug, Display},
    hash::{self, Hash},
    sync::Arc,
};

use ark_ff::fields::PrimeField;

use crate::{asset, Fq};

#[derive(Clone)]
pub struct BaseDenom {
    pub(super) inner: Arc<Inner>,
}

#[derive(Clone)]
pub struct DisplayDenom {
    pub(super) inner: Arc<Inner>,
    // Indexes into the `units` field on `Inner`.
    // The units field is always sorted by priority order.
    pub(super) unit_index: usize,
}

// These are constructed by the asset registry.
pub(super) struct Inner {
    id: asset::Id,
    base_denom: String,
    /// Sorted by priority order.
    pub(super) units: Vec<Unit>,
}

pub(super) struct Unit {
    pub exponent: u8,
    pub denom: String,
}

impl Inner {
    /// Constructs the backing data for a set of units.
    ///
    /// The base denom is added as a unit, so `units` can be empty and should
    /// not include a unit for the base denomination.
    pub fn new(base_denom: String, mut units: Vec<Unit>) -> Self {
        let id = asset::Id(Fq::from_le_bytes_mod_order(
            // XXX choice of hash function?
            blake2b_simd::Params::default()
                .personal(b"Penumbra_AssetID")
                .hash(base_denom.as_bytes())
                .as_bytes(),
        ));

        for unit in &units {
            assert_ne!(unit.exponent, 0);
            assert_ne!(&unit.denom, &base_denom);
        }

        units.push(Unit {
            exponent: 0,
            denom: base_denom.clone(),
        });

        Self {
            id,
            units,
            base_denom,
        }
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
                unit_index,
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

impl From<BaseDenom> for asset::Id {
    fn from(base: BaseDenom) -> asset::Id {
        base.id()
    }
}

impl Hash for BaseDenom {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.inner.base_denom.hash(state);
    }
}

impl PartialEq for BaseDenom {
    fn eq(&self, other: &Self) -> bool {
        self.inner.base_denom.eq(&other.inner.base_denom)
    }
}

impl Eq for BaseDenom {}

impl Debug for BaseDenom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.base_denom.as_str())
    }
}

impl Display for BaseDenom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.base_denom.as_str())
    }
}

impl DisplayDenom {
    pub fn base(&self) -> BaseDenom {
        BaseDenom {
            inner: self.inner.clone(),
        }
    }

    pub fn format_value(&self, value: u64) -> String {
        let power_of_ten = 10u64.pow(self.exponent().into());
        let v1 = value / power_of_ten;
        let v2 = value % power_of_ten;

        // Pad `v2` to exponent digits.
        let v2_str = format!("{:0width$}", v2, width = self.exponent() as usize);
        // For `v2`, there may be trailing zeros that should be stripped
        // since they are after the decimal point.
        let v2_stripped = v2_str.trim_end_matches('0');

        if v2 != 0 {
            format!("{}.{}", v1, v2_stripped)
        } else {
            format!("{}", v1)
        }
    }

    pub fn parse_value(&self, value: &str) -> Result<u64, anyhow::Error> {
        let split: Vec<&str> = value.split(".").collect();
        if split.len() > 2 {
            return Err(anyhow::anyhow!("expected only one decimal point"));
        } else {
            let left = split[0];
            let right;

            // The decimal point and right hand side is optional. If it's not present, we use "0"
            // such that the rest of the logic is the same.
            if split.len() > 1 {
                right = split[1];
            } else {
                right = "0";
            }

            let v1 = left.parse::<u64>().map_err(|e| anyhow::anyhow!(e))?;
            let mut v2 = right.parse::<u64>().map_err(|e| anyhow::anyhow!(e))?;
            let v1_power_of_ten = 10u64.pow(self.exponent().into());

            if right.len() == (self.exponent() + 1) as usize && v2 == 0 {
                // This stanza means that the value is the base unit. Simply return v1.
                return Ok(v1);
            } else if right.len() > self.exponent().into() {
                return Err(anyhow::anyhow!("cannot represent this value"));
            }

            let v2_power_of_ten = 10u64.pow((self.exponent() - right.len() as u8).into());
            v2 = v2.checked_mul(v2_power_of_ten).unwrap();

            let v = v1
                .checked_mul(v1_power_of_ten)
                .and_then(|x| x.checked_add(v2));
            if v.is_none() {
                return Err(anyhow::anyhow!("overflow!"));
            } else {
                return Ok(v.unwrap());
            }
        }
    }

    fn exponent(&self) -> u8 {
        self.inner
            .units
            .get(self.unit_index as usize)
            .expect("there must be an entry for unit_index")
            .exponent
    }
}

impl Hash for DisplayDenom {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.inner.base_denom.hash(state);
        self.unit_index.hash(state);
    }
}

impl PartialEq for DisplayDenom {
    fn eq(&self, other: &Self) -> bool {
        self.inner.base_denom.eq(&other.inner.base_denom) && self.unit_index.eq(&other.unit_index)
    }
}

impl Eq for DisplayDenom {}

impl Debug for DisplayDenom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.units[self.unit_index].denom.as_str())
    }
}

impl Display for DisplayDenom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.units[self.unit_index].denom.as_str())
    }
}
