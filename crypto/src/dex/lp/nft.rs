use penumbra_proto::{core::dex::v1alpha1 as pb, Protobuf};
use regex::Regex;

use super::position::{Id, State};
use crate::asset;

/// The denomination of an LPNFT tracking both ownership and state of a position.
///
/// Tracking the state as part of the LPNFT means that all LP-related actions can
/// be authorized by spending funds: a state transition (e.g., closing a
/// position) is modeled as spending an "open position LPNFT" and minting a
/// "closed position LPNFT" for the same (globally unique) position ID.
///
/// This means that the LP mechanics can be agnostic to the mechanism used to
/// record custody and spend authorization.  For instance, they can be recorded
/// in the shielded pool, where custody is based on off-chain keys, or they could
/// be recorded in a programmatic on-chain account (in the future, e.g., to
/// support interchain accounts).  This also means that LP-related actions don't
/// require any cryptographic implementation (proofs, signatures, etc), other
/// than hooking into the value commitment mechanism used for transaction
/// balances.
#[derive(Debug, Clone)]
pub struct LpNft {
    position_id: Id,
    state: State,
    base_denom: asset::Denom,
}

impl LpNft {
    pub fn new(position_id: Id, state: State) -> Self {
        let base_denom = asset::REGISTRY
            .parse_denom(&format!("lpnft_{}_{}", state, position_id))
            .expect("base denom format is valid");

        Self {
            position_id,
            state,
            base_denom,
        }
    }

    pub fn denom(&self) -> asset::Denom {
        self.base_denom.clone()
    }

    pub fn asset_id(&self) -> asset::Id {
        self.base_denom.id()
    }

    pub fn position_id(&self) -> Id {
        self.position_id.clone()
    }

    pub fn state(&self) -> State {
        self.state.clone()
    }
}

impl TryFrom<asset::Denom> for LpNft {
    type Error = anyhow::Error;

    fn try_from(base_denom: asset::Denom) -> Result<Self, Self::Error> {
        // Note: this regex must be in sync with both asset::REGISTRY
        // and the bech32 prefix for LP IDs defined in the proto crate.
        let base_denom_string = base_denom.to_string();
        let captures = Regex::new("^lpnft_(?P<state>[a-z]+)_(?P<id>plpid1[a-zA-HJ-NP-Z0-9]+)$")
            .expect("regex is valid")
            .captures(&base_denom_string)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "base denom {} is not a delegation token",
                    base_denom.to_string()
                )
            })?;

        let position_id = captures
            .name("id")
            .expect("id is a named capture")
            .as_str()
            .parse()?;
        let state = captures
            .name("state")
            .expect("state is a named capture")
            .as_str()
            .parse()?;

        Ok(Self {
            position_id,
            state,
            base_denom,
        })
    }
}

impl std::fmt::Display for LpNft {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.base_denom.fmt(f)
    }
}

impl std::str::FromStr for LpNft {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let base_denom = asset::REGISTRY
            .parse_denom(s)
            .ok_or_else(|| anyhow::anyhow!("invalid denom string"))?;
        base_denom.try_into()
    }
}

impl std::cmp::PartialEq for LpNft {
    fn eq(&self, other: &Self) -> bool {
        self.position_id == other.position_id && self.state == other.state
    }
}

impl std::cmp::Eq for LpNft {}

impl Protobuf<pb::LpNft> for LpNft {}

impl TryFrom<pb::LpNft> for LpNft {
    type Error = anyhow::Error;

    fn try_from(value: pb::LpNft) -> Result<Self, Self::Error> {
        let position_id = value
            .position_id
            .ok_or_else(|| anyhow::anyhow!("missing position id"))?
            .try_into()?;
        let state = value
            .state
            .ok_or_else(|| anyhow::anyhow!("missing position state"))?
            .try_into()?;

        Ok(Self::new(position_id, state))
    }
}

impl From<LpNft> for pb::LpNft {
    fn from(v: LpNft) -> Self {
        pb::LpNft {
            position_id: Some(v.position_id.into()),
            state: Some(v.state.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::{super::TradingPair, position::*, BareTradingFunction};

    #[test]
    fn lpnft_denom_parsing_roundtrip() {
        // XXX: this construction will have to be tweaked slightly
        // when we fix the TradingPair code to restrict orderings
        let pair = TradingPair {
            asset_1: crate::STAKING_TOKEN_ASSET_ID.clone(),
            asset_2: crate::asset::REGISTRY.parse_denom("cube").unwrap().id(),
        };
        let phi = BareTradingFunction {
            fee: 1,
            p: 1u64.into(),
            q: 1u64.into(),
        };
        let position = Position {
            phi,
            pair,
            nonce: [1u8; 32],
        };
        let position_id = position.id();

        let lpnft1 = LpNft::new(position_id, State::Opened);

        let lpnft1_denom = lpnft1.denom();
        let lpnft1_string = lpnft1.denom().to_string();

        dbg!(&lpnft1);
        dbg!(&lpnft1_denom);
        dbg!(&lpnft1_string);

        assert_eq!(lpnft1.to_string(), lpnft1_string);

        let lpnft2_denom = asset::REGISTRY.parse_denom(&lpnft1_string).unwrap();
        let lpnft2 = LpNft::try_from(lpnft2_denom).unwrap();

        dbg!(&lpnft2);

        assert_eq!(lpnft1, lpnft2);

        let lpnft3: LpNft = lpnft1_string.parse().unwrap();
        assert_eq!(lpnft1, lpnft3);
    }
}
