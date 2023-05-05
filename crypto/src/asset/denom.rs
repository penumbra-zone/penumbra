use std::{
    cmp::{Eq, PartialEq},
    fmt::{Debug, Display},
    hash::{self, Hash},
    sync::Arc,
};

use ark_ff::fields::PrimeField;
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::{asset, Fq, Value};
/// An asset denomination.
///
/// Each denomination has a unique [`asset::Id`] and base unit, and may also
/// have other display units.
#[derive(Serialize, Deserialize, Clone)]
#[serde(try_from = "pb::Denom", into = "pb::Denom")]
pub struct Denom {
    pub(super) inner: Arc<Inner>,
}

impl DomainType for Denom {
    type Proto = pb::Denom;
}

impl From<Denom> for pb::Denom {
    fn from(dn: Denom) -> Self {
        pb::Denom {
            denom: dn.inner.base_denom.clone(),
        }
    }
}

impl TryFrom<pb::Denom> for Denom {
    type Error = anyhow::Error;

    fn try_from(value: pb::Denom) -> Result<Self, Self::Error> {
        asset::REGISTRY
            .parse_denom(&value.denom)
            .ok_or_else(|| anyhow::anyhow!("invalid denomination {}", value.denom))
    }
}

impl TryFrom<&str> for Denom {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        asset::REGISTRY
            .parse_denom(value)
            .ok_or_else(|| anyhow::anyhow!("invalid denomination {}", value))
    }
}

/// A unit of some asset denomination.
#[derive(Clone)]
pub struct Unit {
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
    pub(super) units: Vec<UnitData>,
}

pub(super) struct UnitData {
    pub exponent: u8,
    pub denom: String,
}

impl Inner {
    /// Constructs the backing data for a set of units.
    ///
    /// The base denom is added as a unit, so `units` can be empty and should
    /// not include a unit for the base denomination.
    pub fn new(base_denom: String, mut units: Vec<UnitData>) -> Self {
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

        units.push(UnitData {
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

impl Denom {
    /// Return the [`asset::Id`] associated with this denomination.
    pub fn id(&self) -> asset::Id {
        self.inner.id.clone()
    }

    /// Create a value of this denomination.
    pub fn value(&self, amount: asset::Amount) -> Value {
        Value {
            amount,
            asset_id: self.id(),
        }
    }

    /// Return a list of display units for this denomination, in size order.
    ///
    /// There will always be at least one display denomination.
    pub fn units(&self) -> Vec<Unit> {
        (0..self.inner.units.len())
            .map(|unit_index| Unit {
                unit_index,
                inner: self.inner.clone(),
            })
            .collect()
    }

    /// Returns the default (largest) unit for this denomination.
    pub fn default_unit(&self) -> Unit {
        Unit {
            unit_index: 0,
            inner: self.inner.clone(),
        }
    }

    /// Returns the base (smallest) unit for this denomination.
    ///
    /// (This treats the base denomination as a display unit).
    pub fn base_unit(&self) -> Unit {
        Unit {
            unit_index: self.inner.units.len() - 1,
            inner: self.inner.clone(),
        }
    }

    /// Returns the "best" unit for the given amount (expressed in units of the
    /// base denomination).
    ///
    /// This is defined as the largest unit smaller than the given value (so it
    /// has no leading zeros when formatted).
    pub fn best_unit_for(&self, amount: asset::Amount) -> Unit {
        for (unit_index, unit) in self.inner.units.iter().enumerate() {
            let unit_amount = asset::Amount::from(10u128.pow(unit.exponent as u32));
            if amount >= unit_amount {
                return Unit {
                    unit_index,
                    inner: self.inner.clone(),
                };
            }
        }
        self.base_unit()
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        self.inner.base_denom.starts_with(prefix)
    }
}

impl From<Denom> for asset::Id {
    fn from(base: Denom) -> asset::Id {
        base.id()
    }
}

impl Hash for Denom {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.inner.base_denom.hash(state);
    }
}

impl PartialEq for Denom {
    fn eq(&self, other: &Self) -> bool {
        self.inner.base_denom.eq(&other.inner.base_denom)
    }
}

impl Eq for Denom {}

impl PartialOrd for Denom {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.base_denom.partial_cmp(&other.inner.base_denom)
    }
}

impl Ord for Denom {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.base_denom.cmp(&other.inner.base_denom)
    }
}

impl Debug for Denom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.base_denom.as_str())
    }
}

impl Display for Denom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.base_denom.as_str())
    }
}

impl Unit {
    pub fn base(&self) -> Denom {
        Denom {
            inner: self.inner.clone(),
        }
    }

    /// Return the [`asset::Id`] associated with this denomination.
    pub fn id(&self) -> asset::Id {
        self.inner.id.clone()
    }

    pub fn format_value(&self, value: asset::Amount) -> String {
        let power_of_ten = asset::Amount::from(10u128.pow(self.exponent().into()));
        let v1 = value / power_of_ten;
        let v2 = value % power_of_ten;

        // Pad `v2` to exponent digits.
        let v2_str = format!(
            "{:0width$}",
            u128::from(v2),
            width = self.exponent() as usize
        );

        // For `v2`, there may be trailing zeros that should be stripped
        // since they are after the decimal point.
        let v2_stripped = v2_str.trim_end_matches('0');

        if v2 != asset::Amount::zero() {
            format!("{v1}.{v2_stripped}")
        } else {
            format!("{v1}")
        }
    }

    pub fn parse_value(&self, value: &str) -> Result<asset::Amount, anyhow::Error> {
        let split: Vec<&str> = value.split('.').collect();
        if split.len() > 2 {
            Err(anyhow::anyhow!("expected only one decimal point"))
        } else {
            let left = split[0];

            // The decimal point and right hand side is optional. If it's not present, we use "0"
            // such that the rest of the logic is the same.
            let right = if split.len() > 1 { split[1] } else { "0" };

            let v1 = left.parse::<u128>().map_err(|e| anyhow::anyhow!(e))?;
            let mut v2 = right.parse::<u128>().map_err(|e| anyhow::anyhow!(e))?;
            let v1_power_of_ten = 10u128.pow(self.exponent().into());

            if right.len() == (self.exponent() + 1) as usize && v2 == 0 {
                // This stanza means that the value is the base unit. Simply return v1.
                return Ok(v1.into());
            } else if right.len() > self.exponent().into() {
                return Err(anyhow::anyhow!("cannot represent this value"));
            }

            let v2_power_of_ten = 10u128.pow((self.exponent() - right.len() as u8).into());
            v2 = v2.checked_mul(v2_power_of_ten).unwrap();

            let v = v1
                .checked_mul(v1_power_of_ten)
                .and_then(|x| x.checked_add(v2));

            if let Some(value) = v {
                Ok(value.into())
            } else {
                Err(anyhow::anyhow!("overflow!"))
            }
        }
    }

    pub fn exponent(&self) -> u8 {
        self.inner
            .units
            .get(self.unit_index)
            .expect("there must be an entry for unit_index")
            .exponent
    }

    pub fn unit_amount(&self) -> asset::Amount {
        10u128.pow(self.exponent().into()).into()
    }

    /// Create a value of this unit, applying the correct exponent.
    pub fn value(&self, amount: asset::Amount) -> Value {
        Value {
            asset_id: self.id(),
            amount: amount * self.unit_amount(),
        }
    }
}

impl Hash for Unit {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.inner.base_denom.hash(state);
        self.unit_index.hash(state);
    }
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        self.inner.base_denom.eq(&other.inner.base_denom) && self.unit_index.eq(&other.unit_index)
    }
}

impl Eq for Unit {}

impl Debug for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.units[self.unit_index].denom.as_str())
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.units[self.unit_index].denom.as_str())
    }
}
