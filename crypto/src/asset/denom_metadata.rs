use std::{
    cmp::{Eq, PartialEq},
    fmt::{Debug, Display},
    hash::{self, Hash},
    sync::Arc,
};

use anyhow::ensure;
use ark_ff::fields::PrimeField;
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

use crate::{
    asset::{self},
    Fq, Value,
};

use super::Denom;
/// An asset denomination's metadata.
///
/// Each denomination has a unique [`asset::Id`] and base unit, and may also
/// have other display units.
#[derive(Serialize, Deserialize, Clone)]
#[serde(try_from = "pb::DenomMetadata", into = "pb::DenomMetadata")]
pub struct DenomMetadata {
    pub(super) inner: Arc<Inner>,
}

// These are constructed by the asset registry.
pub(super) struct Inner {
    // The Penumbra asset ID
    id: asset::Id,
    base_denom: String,
    description: String,
    /// Sorted by priority order.
    pub(super) units: Vec<UnitData>,
    //display: String,
    // Indexes into the units array.
    display_index: usize,
    name: String,
    symbol: String,
    uri: String,
    uri_hash: String,
}

impl TypeUrl for DenomMetadata {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.DenomMetadata";
}

impl DomainType for DenomMetadata {
    type Proto = pb::DenomMetadata;
}

impl From<&Inner> for pb::DenomMetadata {
    fn from(inner: &Inner) -> Self {
        pb::DenomMetadata {
            description: inner.description.clone(),
            base: inner.base_denom.clone(),
            display: inner.units[inner.display_index].denom.clone(),
            name: inner.name.clone(),
            symbol: inner.symbol.clone(),
            uri: inner.uri.clone(),
            uri_hash: inner.uri_hash.clone(),
            penumbra_asset_id: Some(inner.id.into()),
            denom_units: inner.units.clone().into_iter().map(|x| x.into()).collect(),
        }
    }
}

impl TryFrom<pb::DenomMetadata> for Inner {
    type Error = anyhow::Error;

    fn try_from(value: pb::DenomMetadata) -> Result<Self, Self::Error> {
        let base_denom = value.base;
        ensure!(
            !base_denom.is_empty(),
            "denom metadata must have a base denom"
        );

        // Compute the ID from the base denom to ensure we don't get confused.
        let id = asset::Id::from_raw_denom(&base_denom);
        // If the ID was supplied, we should check that it's consistent with the base denom.
        if let Some(supplied_id) = value.penumbra_asset_id {
            let supplied_id = asset::Id::try_from(supplied_id)?;
            ensure!(
                id == supplied_id,
                "denom metadata has mismatched penumbra asset ID"
            );
        }

        // Parse the list of units, which may be empty.
        let mut units = value
            .denom_units
            .into_iter()
            .map(UnitData::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        // Ensure that the base denom is present in the unit list.
        // TODO: should we require it to be first?
        if !units.iter().any(|unit| unit.denom == base_denom) {
            units.push(UnitData {
                denom: base_denom.clone(),
                exponent: 0,
            });
        }

        let display_index = if !value.display.is_empty() {
            units
                .iter()
                .position(|unit| unit.denom == value.display)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "display denom {} not found in units {:?}",
                        value.display,
                        units
                    )
                })?
        } else {
            0
        };

        Ok(Inner {
            id,
            base_denom,
            units,
            display_index,
            description: value.description,
            name: value.name,
            symbol: value.symbol,
            uri: value.uri,
            uri_hash: value.uri_hash,
        })
    }
}

impl From<DenomMetadata> for pb::DenomMetadata {
    fn from(dn: DenomMetadata) -> Self {
        dn.inner.as_ref().into()
    }
}

impl TryFrom<pb::DenomMetadata> for DenomMetadata {
    type Error = anyhow::Error;

    fn try_from(value: pb::DenomMetadata) -> Result<Self, Self::Error> {
        let inner = Inner::try_from(value)?;
        Ok(DenomMetadata {
            inner: Arc::new(inner),
        })
    }
}

impl TryFrom<&str> for DenomMetadata {
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

#[derive(Clone, Debug)]
pub(super) struct UnitData {
    pub exponent: u8,
    pub denom: String,
}
impl TryFrom<pb::DenomUnit> for UnitData {
    type Error = anyhow::Error;

    fn try_from(value: pb::DenomUnit) -> Result<Self, Self::Error> {
        Ok(UnitData {
            exponent: value.exponent as u8,
            denom: value.denom,
        })
    }
}
impl From<UnitData> for pb::DenomUnit {
    fn from(dn: UnitData) -> Self {
        pb::DenomUnit {
            denom: dn.denom,
            exponent: dn.exponent as u32,
            aliases: Vec::new(),
        }
    }
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
            display_index: units.len() - 1,
            units,
            base_denom,
            description: String::new(),
            name: String::new(),
            symbol: String::new(),
            uri: String::new(),
            uri_hash: String::new(),
        }
    }
}

impl DenomMetadata {
    /// Return the [`asset::Id`] associated with this denomination.
    pub fn id(&self) -> asset::Id {
        self.inner.id.clone()
    }

    pub fn base_denom(&self) -> Denom {
        Denom {
            denom: self.inner.base_denom.clone(),
        }
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
        // Special case: use the default unit for 0
        if amount == 0u64.into() {
            return self.default_unit();
        }
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

    pub fn default_for(denom: &Denom) -> Option<DenomMetadata> {
        asset::REGISTRY.parse_denom(&denom.denom)
    }

    pub fn is_opened_position_nft(&self) -> bool {
        let prefix = "lpnft_opened_".to_string();

        self.starts_with(&prefix)
    }

    pub fn is_withdrawn_position_nft(&self) -> bool {
        let prefix = "lpnft_withdrawn_".to_string();

        self.starts_with(&prefix)
    }

    pub fn is_closed_position_nft(&self) -> bool {
        let prefix = "lpnft_closed_".to_string();

        self.starts_with(&prefix)
    }
}

impl From<DenomMetadata> for asset::Id {
    fn from(base: DenomMetadata) -> asset::Id {
        base.id()
    }
}

impl Hash for DenomMetadata {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.inner.base_denom.hash(state);
    }
}

impl PartialEq for DenomMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.inner.base_denom.eq(&other.inner.base_denom)
    }
}

impl Eq for DenomMetadata {}

impl PartialOrd for DenomMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.base_denom.partial_cmp(&other.inner.base_denom)
    }
}

impl Ord for DenomMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.base_denom.cmp(&other.inner.base_denom)
    }
}

impl Debug for DenomMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.base_denom.as_str())
    }
}

impl Display for DenomMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.base_denom.as_str())
    }
}

impl Unit {
    pub fn base(&self) -> DenomMetadata {
        DenomMetadata {
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
