use anyhow::anyhow;
use ark_ff::ToConstraintField;
use ark_r1cs_std::prelude::{AllocVar, EqGadget};
use ark_relations::r1cs::SynthesisError;
use decaf377::Fq;
use penumbra_proto::{penumbra::core::component::dex::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use penumbra_asset::asset::{self, AssetIdVar, Unit, REGISTRY};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "pb::DirectedTradingPair", into = "pb::DirectedTradingPair")]
pub struct DirectedTradingPair {
    pub start: asset::Id,
    pub end: asset::Id,
}

impl DirectedTradingPair {
    pub fn new(start: asset::Id, end: asset::Id) -> Self {
        Self { start, end }
    }

    pub fn to_canonical(&self) -> TradingPair {
        TradingPair::new(self.start, self.end)
    }

    pub fn flip(&self) -> DirectedTradingPair {
        DirectedTradingPair {
            start: self.end,
            end: self.start,
        }
    }
}

impl DomainType for DirectedTradingPair {
    type Proto = pb::DirectedTradingPair;
}

impl From<DirectedTradingPair> for pb::DirectedTradingPair {
    fn from(tp: DirectedTradingPair) -> Self {
        Self {
            start: Some(tp.start.into()),
            end: Some(tp.end.into()),
        }
    }
}

impl TryFrom<pb::DirectedTradingPair> for DirectedTradingPair {
    type Error = anyhow::Error;
    fn try_from(tp: pb::DirectedTradingPair) -> anyhow::Result<Self> {
        Ok(Self {
            start: tp
                .start
                .ok_or_else(|| anyhow::anyhow!("missing directed trading pair start"))?
                .try_into()?,
            end: tp
                .end
                .ok_or_else(|| anyhow::anyhow!("missing directed trading pair end"))?
                .try_into()?,
        })
    }
}

impl From<DirectedTradingPair> for TradingPair {
    fn from(pair: DirectedTradingPair) -> Self {
        pair.to_canonical()
    }
}

/// The canonical representation of a tuple of asset [`Id`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "pb::TradingPair", into = "pb::TradingPair")]
pub struct TradingPair {
    pub(crate) asset_1: asset::Id,
    pub(crate) asset_2: asset::Id,
}

impl TradingPair {
    pub fn new(a: asset::Id, b: asset::Id) -> Self {
        if a < b {
            Self {
                asset_1: a,
                asset_2: b,
            }
        } else {
            Self {
                asset_1: b,
                asset_2: a,
            }
        }
    }

    pub fn asset_1(&self) -> asset::Id {
        self.asset_1
    }

    pub fn asset_2(&self) -> asset::Id {
        self.asset_2
    }

    /// Convert the trading pair to bytes.
    pub(crate) fn to_bytes(self) -> [u8; 64] {
        let mut result: [u8; 64] = [0; 64];
        result[0..32].copy_from_slice(&self.asset_1.0.to_bytes());
        result[32..64].copy_from_slice(&self.asset_2.0.to_bytes());
        result
    }
}

impl TryFrom<[u8; 64]> for TradingPair {
    type Error = anyhow::Error;
    fn try_from(bytes: [u8; 64]) -> anyhow::Result<Self> {
        let asset_1_bytes = &bytes[0..32];
        let asset_2_bytes = &bytes[32..64];
        let asset_1 = asset_1_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid asset_1 bytes in TradingPair"))?;
        let asset_2 = asset_2_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid asset_2 bytes in TradingPair"))?;
        let trading_pair = TradingPair::new(asset_1, asset_2);
        let result = Self { asset_1, asset_2 };
        if trading_pair != result {
            anyhow::bail!("non-canonical trading pair");
        }
        Ok(result)
    }
}

/// Represents a trading pair in R1CS.
pub struct TradingPairVar {
    pub asset_1: asset::AssetIdVar,
    pub asset_2: asset::AssetIdVar,
}

impl TradingPairVar {
    /// This allocates a `TradingPairVar` without checking it is canonically ordered.
    ///
    /// # Safety
    ///
    /// Does _not_ ensure that asset_1 <= asset_2, as is the case for `TradingPair`.
    pub fn new_variable_unchecked(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<TradingPair, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let trading_pair = f()?;
        let asset_1 = AssetIdVar::new_variable(cs.clone(), || Ok(trading_pair.asset_1()), mode)?;
        let asset_2 = AssetIdVar::new_variable(cs, || Ok(trading_pair.asset_2()), mode)?;
        // Note: We do not check that the trading pair is canonically encoded.
        Ok(Self { asset_1, asset_2 })
    }
}

impl ToConstraintField<Fq> for TradingPair {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        let mut public_inputs = Vec::new();
        public_inputs.extend(
            Fq::from(self.asset_1().0)
                .to_field_elements()
                .expect("Fq types are Bls12-377 field members"),
        );
        public_inputs.extend(
            Fq::from(self.asset_2().0)
                .to_field_elements()
                .expect("Fq types are Bls12-377 field members"),
        );
        Some(public_inputs)
    }
}

impl EqGadget<Fq> for TradingPairVar {
    fn is_eq(&self, other: &Self) -> Result<ark_r1cs_std::prelude::Boolean<Fq>, SynthesisError> {
        let asset_1_eq = self.asset_1.is_eq(&other.asset_1);
        let asset_2_eq = self.asset_2.is_eq(&other.asset_2);
        asset_1_eq.and(asset_2_eq)
    }
}

impl DomainType for TradingPair {
    type Proto = pb::TradingPair;
}

impl From<TradingPair> for pb::TradingPair {
    fn from(tp: TradingPair) -> Self {
        Self {
            asset_1: Some(tp.asset_1.into()),
            asset_2: Some(tp.asset_2.into()),
        }
    }
}

impl TryFrom<pb::TradingPair> for TradingPair {
    type Error = anyhow::Error;
    fn try_from(tp: pb::TradingPair) -> anyhow::Result<Self> {
        let asset_1 = tp
            .asset_1
            .ok_or_else(|| anyhow::anyhow!("missing trading pair asset1"))?
            .try_into()?;
        let asset_2 = tp
            .asset_2
            .ok_or_else(|| anyhow::anyhow!("missing trading pair asset2"))?
            .try_into()?;
        let trading_pair = TradingPair::new(asset_1, asset_2);
        Ok(trading_pair)
    }
}

impl FromStr for TradingPair {
    type Err = anyhow::Error;

    /// Takes an input of the form DENOM1:DENOM2,
    /// splits on the `:` (erroring if there is more than one `:`),
    /// parses the first and second halves using `asset::REGISTRY.parse_unit`,
    /// then computes the asset IDs and then the canonically-ordered trading pair.
    fn from_str(s: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() != 2 {
            Err(anyhow!("invalid trading pair string"))
        } else {
            let denom_1 = REGISTRY.parse_unit(parts[0]);
            let denom_2 = REGISTRY.parse_unit(parts[1]);
            Ok(Self::new(denom_1.id(), denom_2.id()))
        }
    }
}

impl FromStr for DirectedTradingPair {
    type Err = anyhow::Error;

    /// Takes an input of the form DENOM1:DENOM2,
    /// splits on the `:` (erroring if there is more than one `:`),
    /// parses the first and second halves using `asset::REGISTRY.parse_unit`,
    /// then computes the asset IDs and then the canonically-ordered trading pair.
    fn from_str(s: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() != 2 {
            Err(anyhow!("invalid trading pair string"))
        } else {
            let denom_1 = REGISTRY.parse_unit(parts[0]);
            let denom_2 = REGISTRY.parse_unit(parts[1]);
            Ok(Self::new(denom_1.id(), denom_2.id()))
        }
    }
}

/// Produce an output string of the form ASSET_ID1:ASSET_ID2
/// TODO: this mismatches the `FromStr` impl which uses denominations.
/// The asset ID is more canonical than a base denom so I think that's okay,
/// but the `FromStr` impl should probably be able to handle asset IDs as well.
impl fmt::Display for TradingPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.asset_1, self.asset_2)
    }
}

/// A directed tuple of `Unit`s, similar to a `DirectedTradingPair` but embedding
/// useful denom data.
#[derive(Clone, Debug)]
pub struct DirectedUnitPair {
    pub start: Unit,
    pub end: Unit,
}

impl DirectedUnitPair {
    pub fn new(start: Unit, end: Unit) -> Self {
        Self { start, end }
    }

    pub fn to_canonical_string(&self) -> String {
        if self.match_canonical_ordering() {
            self.to_string()
        } else {
            self.flip().to_string()
        }
    }

    pub fn match_canonical_ordering(&self) -> bool {
        self.start.id() > self.end.id()
    }

    pub fn into_directed_trading_pair(&self) -> DirectedTradingPair {
        DirectedTradingPair {
            start: self.start.id(),
            end: self.end.id(),
        }
    }

    pub fn flip(&self) -> Self {
        DirectedUnitPair {
            start: self.end.clone(),
            end: self.start.clone(),
        }
    }
}

impl FromStr for DirectedUnitPair {
    type Err = anyhow::Error;

    /// Takes an input of the form DENOM1:DENOM2,
    /// splits on the `:` (erroring if there is more than one `:`),
    /// parses the first and second halves using `asset::REGISTRY.parse_unit`,
    /// then forms a `Market` i.e. which is a directed tuple of `Units`.
    fn from_str(s: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() != 2 {
            Err(anyhow!("invalid market string"))
        } else {
            let denom_1 = REGISTRY.parse_unit(parts[0]);
            let denom_2 = REGISTRY.parse_unit(parts[1]);
            Ok(Self::new(denom_1, denom_2))
        }
    }
}

impl Display for DirectedUnitPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.start, self.end)
    }
}
