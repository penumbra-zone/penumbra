//! Values (?)

use ark_ff::ToConstraintField;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{r1cs::FqVar, Fq};

use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

use anyhow::Context;
use penumbra_sdk_num::{Amount, AmountVar};
use penumbra_sdk_proto::{penumbra::core::asset::v1 as pb, DomainType};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::EquivalentValue;
use crate::{
    asset::{AssetIdVar, Cache, Id, Metadata, REGISTRY},
    EstimatedPrice,
};

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "pb::Value", into = "pb::Value")]
pub struct Value {
    pub amount: Amount,
    // The asset ID. 256 bits.
    pub asset_id: Id,
}

/// Represents a value of a known or unknown denomination.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(try_from = "pb::ValueView", into = "pb::ValueView")]
pub enum ValueView {
    KnownAssetId {
        amount: Amount,
        metadata: Metadata,
        equivalent_values: Vec<EquivalentValue>,
        extended_metadata: Option<pbjson_types::Any>,
    },
    UnknownAssetId {
        amount: Amount,
        asset_id: Id,
    },
}

impl ValueView {
    /// Convert this `ValueView` down to the underlying `Value`.
    pub fn value(&self) -> Value {
        self.clone().into()
    }

    /// Get the `Id` of the underlying `Value`, without having to match on visibility.
    pub fn asset_id(&self) -> Id {
        self.value().asset_id
    }

    /// Use the provided [`EstimatedPrice`]s and asset metadata [`Cache`] to add
    /// equivalent values to this [`ValueView`].
    pub fn with_prices(mut self, prices: &[EstimatedPrice], known_metadata: &Cache) -> Self {
        if let ValueView::KnownAssetId {
            ref mut equivalent_values,
            metadata,
            amount,
            ..
        } = &mut self
        {
            // Set the equivalent values.
            *equivalent_values = prices
                .iter()
                .filter_map(|price| {
                    if metadata.id() == price.priced_asset
                        && known_metadata.contains_key(&price.numeraire)
                    {
                        let equivalent_amount_f =
                            (amount.value() as f64) * price.numeraire_per_unit;
                        Some(EquivalentValue {
                            equivalent_amount: Amount::from(equivalent_amount_f as u128),
                            numeraire: known_metadata
                                .get(&price.numeraire)
                                .expect("we checked containment above")
                                .clone(),
                            as_of_height: price.as_of_height,
                        })
                    } else {
                        None
                    }
                })
                .collect();
        }

        self
    }

    /// Use the provided extended metadata to add extended metadata to this [`ValueView`].
    pub fn with_extended_metadata(mut self, extended: Option<pbjson_types::Any>) -> Self {
        if let ValueView::KnownAssetId {
            ref mut extended_metadata,
            ..
        } = &mut self
        {
            *extended_metadata = extended;
        }

        self
    }
}

impl Value {
    /// Convert this `Value` into a `ValueView` with the given `Denom`.
    pub fn view_with_denom(&self, denom: Metadata) -> anyhow::Result<ValueView> {
        if self.asset_id == denom.id() {
            Ok(ValueView::KnownAssetId {
                amount: self.amount,
                metadata: denom,
                equivalent_values: Vec::new(),
                extended_metadata: None,
            })
        } else {
            Err(anyhow::anyhow!(
                "asset ID {} does not match denom {}",
                self.asset_id,
                denom
            ))
        }
    }

    /// Convert this `Value` into a `ValueView` using the given `Cache`
    pub fn view_with_cache(&self, cache: &Cache) -> ValueView {
        match cache.get(&self.asset_id) {
            Some(denom) => ValueView::KnownAssetId {
                amount: self.amount,
                metadata: denom.clone(),
                equivalent_values: Vec::new(),
                extended_metadata: None,
            },
            None => ValueView::UnknownAssetId {
                amount: self.amount,
                asset_id: self.asset_id,
            },
        }
    }
}

impl From<ValueView> for Value {
    fn from(value: ValueView) -> Self {
        match value {
            ValueView::KnownAssetId {
                amount,
                metadata: denom,
                ..
            } => Value {
                amount,
                asset_id: Id::from(denom),
            },
            ValueView::UnknownAssetId { amount, asset_id } => Value { amount, asset_id },
        }
    }
}

impl DomainType for Value {
    type Proto = pb::Value;
}

impl DomainType for ValueView {
    type Proto = pb::ValueView;
}

impl From<Value> for pb::Value {
    fn from(v: Value) -> Self {
        pb::Value {
            amount: Some(v.amount.into()),
            asset_id: Some(v.asset_id.into()),
        }
    }
}

impl TryFrom<pb::Value> for Value {
    type Error = anyhow::Error;
    fn try_from(value: pb::Value) -> Result<Self, Self::Error> {
        Ok(Value {
            amount: value
                .amount
                .ok_or_else(|| {
                    anyhow::anyhow!("could not deserialize Value: missing amount field")
                })?
                .try_into()?,
            asset_id: value
                .asset_id
                .ok_or_else(|| anyhow::anyhow!("missing balance commitment"))?
                .try_into()?,
        })
    }
}

impl From<ValueView> for pb::ValueView {
    fn from(v: ValueView) -> Self {
        match v {
            ValueView::KnownAssetId {
                amount,
                metadata,
                equivalent_values,
                extended_metadata,
            } => pb::ValueView {
                value_view: Some(pb::value_view::ValueView::KnownAssetId(
                    pb::value_view::KnownAssetId {
                        amount: Some(amount.into()),
                        metadata: Some(metadata.into()),
                        equivalent_values: equivalent_values.into_iter().map(Into::into).collect(),
                        extended_metadata,
                    },
                )),
            },
            ValueView::UnknownAssetId { amount, asset_id } => pb::ValueView {
                value_view: Some(pb::value_view::ValueView::UnknownAssetId(
                    pb::value_view::UnknownAssetId {
                        amount: Some(amount.into()),
                        asset_id: Some(asset_id.into()),
                    },
                )),
            },
        }
    }
}

impl TryFrom<pb::ValueView> for ValueView {
    type Error = anyhow::Error;
    fn try_from(value: pb::ValueView) -> Result<Self, Self::Error> {
        match value
            .value_view
            .ok_or_else(|| anyhow::anyhow!("missing value_view field"))?
        {
            pb::value_view::ValueView::KnownAssetId(v) => Ok(ValueView::KnownAssetId {
                amount: v
                    .amount
                    .ok_or_else(|| anyhow::anyhow!("missing amount field"))?
                    .try_into()?,
                metadata: v
                    .metadata
                    .ok_or_else(|| anyhow::anyhow!("missing denom field"))?
                    .try_into()?,
                equivalent_values: v
                    .equivalent_values
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                extended_metadata: v.extended_metadata,
            }),
            pb::value_view::ValueView::UnknownAssetId(v) => Ok(ValueView::UnknownAssetId {
                amount: v
                    .amount
                    .ok_or_else(|| anyhow::anyhow!("missing amount field"))?
                    .try_into()?,
                asset_id: v
                    .asset_id
                    .ok_or_else(|| anyhow::anyhow!("missing asset_id field"))?
                    .try_into()?,
            }),
        }
    }
}

impl Value {
    /// Use the provided [`Cache`] to format this value.
    ///
    /// Returns the amount in terms of the asset ID if the denomination is not known.
    pub fn format(&self, cache: &Cache) -> String {
        cache
            .get(&self.asset_id)
            .map(|base_denom| {
                let display_denom = base_denom.best_unit_for(self.amount);
                format!(
                    "{}{}",
                    display_denom.format_value(self.amount),
                    display_denom
                )
            })
            .unwrap_or_else(|| format!("{}{}", self.amount, self.asset_id))
    }
}

#[derive(Clone)]
pub struct ValueVar {
    pub amount: AmountVar,
    pub asset_id: AssetIdVar,
}

impl AllocVar<Value, Fq> for ValueVar {
    fn new_variable<T: std::borrow::Borrow<Value>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner: Value = *f()?.borrow();

        let amount_var = AmountVar::new_variable(cs.clone(), || Ok(inner.amount), mode)?;
        let asset_id_var = AssetIdVar::new_variable(cs, || Ok(inner.asset_id), mode)?;
        Ok(Self {
            amount: amount_var,
            asset_id: asset_id_var,
        })
    }
}

impl ToConstraintField<Fq> for Value {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        let mut elements = Vec::new();
        elements.extend_from_slice(&self.amount.to_field_elements()?);
        elements.extend_from_slice(&self.asset_id.to_field_elements()?);
        Some(elements)
    }
}

impl EqGadget<Fq> for ValueVar {
    fn is_eq(&self, other: &Self) -> Result<Boolean<Fq>, SynthesisError> {
        let amount_eq = self.amount.is_eq(&other.amount)?;
        let asset_id_eq = self.asset_id.is_eq(&other.asset_id)?;
        amount_eq.and(&asset_id_eq)
    }
}

impl R1CSVar<Fq> for ValueVar {
    type Value = Value;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.amount.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        Ok(Value {
            amount: self.amount.value()?,
            asset_id: self.asset_id.value()?,
        })
    }
}

impl ValueVar {
    pub fn amount(&self) -> FqVar {
        self.amount.amount.clone()
    }

    pub fn negate(&self) -> Result<ValueVar, SynthesisError> {
        Ok(ValueVar {
            amount: self.amount.negate()?,
            asset_id: self.asset_id.clone(),
        })
    }

    pub fn asset_id(&self) -> FqVar {
        self.asset_id.asset_id.clone()
    }
}

impl FromStr for Value {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let asset_id_re =
            Regex::new(r"^([0-9.]+)(passet[0-9].*)$").context("unable to parse asset ID regex")?;
        let denom_re =
            Regex::new(r"^([0-9.]+)([^0-9.].*)$").context("unable to parse denom regex")?;

        if let Some(captures) = asset_id_re.captures(s) {
            let numeric_str = captures
                .get(1)
                .context("string value should have numeric part")?
                .as_str();
            let asset_id_str = captures
                .get(2)
                .context("string value should have asset ID part")?
                .as_str();

            let asset_id =
                Id::from_str(asset_id_str).context("unable to parse string value's asset ID")?;
            let amount = numeric_str
                .parse::<u64>()
                .context("unable to parse string value's numeric amount")?;

            Ok(Value {
                amount: amount.into(),
                asset_id,
            })
        } else if let Some(captures) = denom_re.captures(s) {
            let numeric_str = captures
                .get(1)
                .context("string value should have numeric part")?
                .as_str();
            let denom_str = captures
                .get(2)
                .context("string value should have denom part")?
                .as_str();

            let display_denom = REGISTRY.parse_unit(denom_str);
            let amount = display_denom.parse_value(numeric_str)?;
            let asset_id = display_denom.base().id();

            Ok(Value { amount, asset_id })
        } else {
            Err(anyhow::anyhow!(
                "could not parse {} as a value; provide both a numeric value and denomination, e.g. 1penumbra",
                s
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use decaf377::Fr;
    use std::ops::Deref;

    use crate::{balance::commitment::VALUE_BLINDING_GENERATOR, Balance};

    use super::*;

    #[test]
    fn sum_balance_commitments() {
        let pen_denom = crate::asset::Cache::with_known_assets()
            .get_unit("upenumbra")
            .unwrap()
            .base();
        let atom_denom = crate::asset::Cache::with_known_assets()
            .get_unit("utest_atom")
            .unwrap()
            .base();

        let pen_id = Id::from(pen_denom);
        let atom_id = Id::from(atom_denom);

        // some values of different types
        let v1 = Value {
            amount: 10u64.into(),
            asset_id: pen_id,
        };
        let v2 = Value {
            amount: 8u64.into(),
            asset_id: pen_id,
        };
        let v3 = Value {
            amount: 2u64.into(),
            asset_id: pen_id,
        };
        let v4 = Value {
            amount: 13u64.into(),
            asset_id: atom_id,
        };
        let v5 = Value {
            amount: 17u64.into(),
            asset_id: atom_id,
        };
        let v6 = Value {
            amount: 30u64.into(),
            asset_id: atom_id,
        };

        // some random-looking blinding factors
        let b1 = Fr::from(129u64).inverse().unwrap();
        let b2 = Fr::from(199u64).inverse().unwrap();
        let b3 = Fr::from(121u64).inverse().unwrap();
        let b4 = Fr::from(179u64).inverse().unwrap();
        let b5 = Fr::from(379u64).inverse().unwrap();
        let b6 = Fr::from(879u64).inverse().unwrap();

        // form commitments
        let c1 = v1.commit(b1);
        let c2 = v2.commit(b2);
        let c3 = v3.commit(b3);
        let c4 = v4.commit(b4);
        let c5 = v5.commit(b5);
        let c6 = v6.commit(b6);

        // values sum to 0, so this is a commitment to 0...
        let c0 = c1 - c2 - c3 + c4 + c5 - c6;
        // with the following synthetic blinding factor:
        let b0 = b1 - b2 - b3 + b4 + b5 - b6;

        // so c0 = 0 * G_v1 + 0 * G_v2 + b0 * H
        assert_eq!(c0.0, b0 * VALUE_BLINDING_GENERATOR.deref());

        // Now we do the same, but using the `Balance` structure.
        let balance1 = Balance::from(v1);
        let balance2 = Balance::from(v2);
        let balance3 = Balance::from(v3);
        let balance4 = Balance::from(v4);
        let balance5 = Balance::from(v5);
        let balance6 = Balance::from(v6);

        let balance_total = balance1 - balance2 - balance3 + balance4 + balance5 - balance6;
        assert_eq!(balance_total.commit(b0), c0);
        // The commitment derived from the `Balance` structure is equivalent to `c0` when it was
        // computed using the summed synthetic blinding factor `b0`, where we took care to use the
        // same signs.
    }

    #[test]
    fn value_parsing_happy() {
        let upenumbra_sdk_base_denom = crate::asset::Cache::with_known_assets()
            .get_unit("upenumbra")
            .unwrap()
            .base();
        let nala_base_denom = crate::asset::Cache::with_known_assets()
            .get_unit("unala")
            .unwrap()
            .base();
        let cache = [upenumbra_sdk_base_denom.clone(), nala_base_denom.clone()]
            .into_iter()
            .collect::<Cache>();

        let v1: Value = "1823.298penumbra".parse().unwrap();
        assert_eq!(v1.amount, 1823298000u64.into());
        assert_eq!(v1.asset_id, upenumbra_sdk_base_denom.id());
        // Check that we can also parse the output of try_format
        assert_eq!(v1, v1.format(&cache).parse().unwrap());

        let v2: Value = "3930upenumbra".parse().unwrap();
        assert_eq!(v2.amount, 3930u64.into());
        assert_eq!(v2.asset_id, upenumbra_sdk_base_denom.id());
        assert_eq!(v2, v2.format(&cache).parse().unwrap());

        let v1: Value = "1unala".parse().unwrap();
        assert_eq!(v1.amount, 1u64.into());
        assert_eq!(v1.asset_id, nala_base_denom.id());
        assert_eq!(v1, v1.format(&cache).parse().unwrap());
    }

    #[test]
    fn value_parsing_errors() {
        assert!(Value::from_str("1").is_err());
        assert!(Value::from_str("nala").is_err());
    }

    #[test]
    fn format_picks_best_unit() {
        let upenumbra_sdk_base_denom = crate::asset::Cache::with_known_assets()
            .get_unit("upenumbra")
            .unwrap()
            .base();
        let cache = [upenumbra_sdk_base_denom].into_iter().collect::<Cache>();

        let v1: Value = "999upenumbra".parse().unwrap();
        let v2: Value = "1000upenumbra".parse().unwrap();
        let v3: Value = "4000000upenumbra".parse().unwrap();

        assert_eq!(v1.format(&cache), "999upenumbra");
        assert_eq!(v2.format(&cache), "1mpenumbra");
        assert_eq!(v3.format(&cache), "4penumbra");
    }
}
