use std::str::FromStr;

use regex::Regex;

use penumbra_sdk_asset::asset;

use crate::IdentityKey;

/// Unbonding tokens represent staking tokens that are currently unbonding and
/// subject to slashing.
///
/// Unbonding tokens are parameterized by the validator identity, and the epoch at
/// which unbonding began.
pub struct UnbondingToken {
    validator_identity: IdentityKey,
    unbonding_start_height: u64,
    base_denom: asset::Metadata,
}

impl UnbondingToken {
    pub fn new(validator_identity: IdentityKey, unbonding_start_height: u64) -> Self {
        // This format string needs to be in sync with the asset registry
        let base_denom = asset::REGISTRY
            .parse_denom(&format!(
                // "uu" is not a typo, these are micro-unbonding tokens
                "uunbonding_start_at_{unbonding_start_height}_{validator_identity}"
            ))
            .expect("base denom format is valid");
        UnbondingToken {
            validator_identity,
            base_denom,
            unbonding_start_height,
        }
    }

    /// Get the base denomination for this unbonding token.
    pub fn denom(&self) -> asset::Metadata {
        self.base_denom.clone()
    }

    /// Get the default display denomination for this unbonding token.
    pub fn default_unit(&self) -> asset::Unit {
        self.base_denom.default_unit()
    }

    /// Get the asset ID for this unbonding token.
    pub fn id(&self) -> asset::Id {
        self.base_denom.id()
    }

    /// Get the identity key of the validator this unbonding token is associated with.
    pub fn validator(&self) -> IdentityKey {
        self.validator_identity.clone()
    }

    pub fn unbonding_start_height(&self) -> u64 {
        self.unbonding_start_height
    }
}

impl TryFrom<asset::Metadata> for UnbondingToken {
    type Error = anyhow::Error;

    fn try_from(base_denom: asset::Metadata) -> Result<Self, Self::Error> {
        let base_string = base_denom.to_string();

        // Note: this regex must be in sync with both asset::REGISTRY
        // and VALIDATOR_IDENTITY_BECH32_PREFIX
        // The data capture group is used by asset::REGISTRY
        let captures =
            Regex::new("^uunbonding_(?P<data>start_at_(?P<start>[0-9]+)_(?P<validator>penumbravalid1[a-zA-HJ-NP-Z0-9]+))$")
                .expect("regex is valid")
                .captures(base_string.as_ref())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "base denom {} is not an unbonding token",
                        base_denom.to_string()
                    )
                })?;

        let validator_identity = captures
            .name("validator")
            .expect("validator is a named capture")
            .as_str()
            .parse()?;

        let unbonding_start_height = captures
            .name("start")
            .expect("start is a named capture")
            .as_str()
            .parse()?;

        Ok(Self {
            base_denom,
            validator_identity,
            unbonding_start_height,
        })
    }
}

impl FromStr for UnbondingToken {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        asset::REGISTRY
            .parse_denom(s)
            .ok_or_else(|| anyhow::anyhow!("could not parse {} as base denomination", s))?
            .try_into()
    }
}

impl std::fmt::Display for UnbondingToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base_denom.fmt(f)
    }
}

impl std::fmt::Debug for UnbondingToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base_denom.fmt(f)
    }
}

impl PartialEq for UnbondingToken {
    fn eq(&self, other: &Self) -> bool {
        self.base_denom.eq(&other.base_denom)
    }
}

impl Eq for UnbondingToken {}

impl std::hash::Hash for UnbondingToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.base_denom.hash(state)
    }
}

#[cfg(test)]
mod tests {
    use decaf377_rdsa::{SigningKey, VerificationKey};

    use super::*;

    #[test]
    fn unbonding_token_denomination_round_trip() {
        use rand_core::OsRng;

        let vk = VerificationKey::from(SigningKey::new(OsRng));
        let ik = IdentityKey(vk.into());
        let start = 782;

        let token = UnbondingToken::new(ik, start);

        let denom = token.to_string();
        println!("denom: {denom}");
        let token2 = UnbondingToken::from_str(&denom).unwrap();
        let denom2 = token2.to_string();

        assert_eq!(denom, denom2);
        assert_eq!(token, token2);
    }
}
