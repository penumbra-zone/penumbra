//! Values (?)

use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

use penumbra_proto::core::crypto::v1alpha1 as pb;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::asset;

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "pb::Value", into = "pb::Value")]
pub struct Value {
    pub amount: asset::Amount,
    // The asset ID. 256 bits.
    pub asset_id: asset::Id,
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
                .ok_or_else(|| anyhow::anyhow!("missing value commitment"))?
                .try_into()?,
        })
    }
}

impl Value {
    /// Use the provided [`asset::Cache`] to format this value.
    ///
    /// Returns the amount in terms of the asset ID if the denomination is not known.
    pub fn format(&self, cache: &asset::Cache) -> String {
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

impl FromStr for Value {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let asset_id_re = Regex::new(r"^([0-9.]+)(passet[0-9].*)$").unwrap();
        let denom_re = Regex::new(r"^([0-9.]+)([^0-9.].*)$").unwrap();

        if let Some(captures) = asset_id_re.captures(s) {
            let numeric_str = captures.get(1).expect("matched regex").as_str();
            let asset_id_str = captures.get(2).expect("matched regex").as_str();

            let asset_id = asset::Id::from_str(asset_id_str).expect("able to parse asset ID");
            let amount = numeric_str.parse::<u64>().unwrap();

            Ok(Value {
                amount: amount.into(),
                asset_id,
            })
        } else if let Some(captures) = denom_re.captures(s) {
            let numeric_str = captures.get(1).expect("matched regex").as_str();
            let denom_str = captures.get(2).expect("matched regex").as_str();

            let display_denom = asset::REGISTRY.parse_unit(denom_str);
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

    use crate::{
        balance::commitment::VALUE_BLINDING_GENERATOR,
        dex::{swap::SwapPlaintext, TradingPair},
        transaction::Fee,
        Address, Balance,
    };

    use super::*;

    #[test]
    fn sum_balance_commitments() {
        use ark_ff::Field;

        let pen_denom = asset::REGISTRY.parse_denom("upenumbra").unwrap();
        let atom_denom = asset::REGISTRY
            .parse_denom("HubPort/HubChannel/uatom")
            .unwrap();

        let pen_id = asset::Id::from(pen_denom);
        let atom_id = asset::Id::from(atom_denom);

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
        let b1 = Fr::from(-129).inverse().unwrap();
        let b2 = Fr::from(-199).inverse().unwrap();
        let b3 = Fr::from(-121).inverse().unwrap();
        let b4 = Fr::from(-179).inverse().unwrap();
        let b5 = Fr::from(-379).inverse().unwrap();
        let b6 = Fr::from(-879).inverse().unwrap();

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
        let upenumbra_base_denom = asset::REGISTRY.parse_denom("upenumbra").unwrap();
        let nala_base_denom = asset::REGISTRY.parse_denom("nala").unwrap();
        let cache = [upenumbra_base_denom.clone(), nala_base_denom.clone()]
            .into_iter()
            .collect::<asset::Cache>();

        let v1: Value = "1823.298penumbra".parse().unwrap();
        assert_eq!(v1.amount, 1823298000u64.into());
        assert_eq!(v1.asset_id, upenumbra_base_denom.id());
        // Check that we can also parse the output of try_format
        assert_eq!(v1, v1.format(&cache).parse().unwrap());

        let v2: Value = "3930upenumbra".parse().unwrap();
        assert_eq!(v2.amount, 3930u64.into());
        assert_eq!(v2.asset_id, upenumbra_base_denom.id());
        assert_eq!(v2, v2.format(&cache).parse().unwrap());

        let v1: Value = "1nala".parse().unwrap();
        assert_eq!(v1.amount, 1u64.into());
        assert_eq!(v1.asset_id, nala_base_denom.id());
        assert_eq!(v1, v1.format(&cache).parse().unwrap());

        // Swap NFTs have no associated denom, make sure we can roundtrip parse/format.
        let gm_base_denom = asset::REGISTRY.parse_denom("ugm").unwrap();
        let sp = SwapPlaintext::new(
            TradingPair::new(
                asset::Id::from(gm_base_denom),
                asset::Id::from(upenumbra_base_denom),
            ).unwrap(),
            1u64.into(),
            0u64.into(),
            Fee::default(),
            Address::from_str("penumbrav2t13vh0fkf3qkqjacpm59g23ufea9n5us45e4p5h6hty8vg73r2t8g5l3kynad87u0n9eragf3hhkgkhqe5vhngq2cw493k48c9qg9ms4epllcmndd6ly4v4dw2jcnxaxzjqnlvnw").unwrap()
        );
        let v3: Value = Value {
            amount: 1u64.into(),
            asset_id: sp.asset_id(),
        };
        let asset_id = v3.format(&cache);
        assert_eq!(v3, asset_id.parse().unwrap());
    }

    #[test]
    fn value_parsing_errors() {
        assert!(Value::from_str("1").is_err());
        assert!(Value::from_str("nala").is_err());
    }

    #[test]
    fn format_picks_best_unit() {
        let upenumbra_base_denom = asset::REGISTRY.parse_denom("upenumbra").unwrap();
        let gm_base_denom = asset::REGISTRY.parse_denom("ugm").unwrap();
        let cache = [upenumbra_base_denom.clone()]
            .into_iter()
            .collect::<asset::Cache>();

        let v1: Value = "999upenumbra".parse().unwrap();
        let v2: Value = "1000upenumbra".parse().unwrap();
        let v3: Value = "4000000upenumbra".parse().unwrap();
        // Swap NFTs have no associated denom, make sure the formatter doesn't blow up.
        let sp = SwapPlaintext::new(
            TradingPair::new(
                asset::Id::from(gm_base_denom),
                asset::Id::from(upenumbra_base_denom),
            ).unwrap(),
            1u64.into(),
            0u64.into(),
            Fee::default(),
            Address::from_str("penumbrav2t13vh0fkf3qkqjacpm59g23ufea9n5us45e4p5h6hty8vg73r2t8g5l3kynad87u0n9eragf3hhkgkhqe5vhngq2cw493k48c9qg9ms4epllcmndd6ly4v4dw2jcnxaxzjqnlvnw").unwrap()
        );
        let v4: Value = Value {
            amount: 1u64.into(),
            asset_id: sp.asset_id(),
        };

        assert_eq!(v1.format(&cache), "999upenumbra");
        assert_eq!(v2.format(&cache), "1mpenumbra");
        assert_eq!(v3.format(&cache), "4penumbra");
        assert_eq!(&v4.format(&cache)[..8], "1passet1");
    }
}
