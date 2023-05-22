use std::str::FromStr;

use regex::Regex;

use super::IdentityKey;
use crate::asset;

/// Delegation tokens represent a share of a particular validator's delegation pool.
pub struct DelegationToken {
    validator_identity: IdentityKey,
    base_denom: asset::DenomMetadata,
}

impl From<IdentityKey> for DelegationToken {
    fn from(v: IdentityKey) -> Self {
        DelegationToken::new(v)
    }
}

impl From<&IdentityKey> for DelegationToken {
    fn from(v: &IdentityKey) -> Self {
        DelegationToken::new(v.clone())
    }
}

impl DelegationToken {
    pub fn new(validator_identity: IdentityKey) -> Self {
        // This format string needs to be in sync with the asset registry
        let base_denom = asset::REGISTRY
            .parse_denom(&format!("udelegation_{validator_identity}"))
            .expect("base denom format is valid");
        DelegationToken {
            validator_identity,
            base_denom,
        }
    }

    /// Get the base denomination for this delegation token.
    pub fn denom(&self) -> asset::DenomMetadata {
        self.base_denom.clone()
    }

    /// Get the default display denomination for this delegation token.
    pub fn default_unit(&self) -> asset::Unit {
        self.base_denom.default_unit()
    }

    /// Get the asset ID for this delegation token.
    pub fn id(&self) -> asset::Id {
        self.base_denom.id()
    }

    /// Get the identity key of the validator this delegation token is associated with.
    pub fn validator(&self) -> IdentityKey {
        self.validator_identity.clone()
    }
}

impl TryFrom<asset::DenomMetadata> for DelegationToken {
    type Error = anyhow::Error;
    fn try_from(base_denom: asset::DenomMetadata) -> Result<Self, Self::Error> {
        // Note: this regex must be in sync with both asset::REGISTRY
        // and VALIDATOR_IDENTITY_BECH32_PREFIX
        let validator_identity =
            Regex::new("udelegation_(?P<data>penumbravalid1[a-zA-HJ-NP-Z0-9]+)")
                .expect("regex is valid")
                .captures(&base_denom.to_string())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "base denom {} is not a delegation token",
                        base_denom.to_string()
                    )
                })?
                .name("data")
                .expect("data is a named capture")
                .as_str()
                .parse()?;

        Ok(Self {
            base_denom,
            validator_identity,
        })
    }
}

impl FromStr for DelegationToken {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        asset::REGISTRY
            .parse_denom(s)
            .ok_or_else(|| anyhow::anyhow!("could not parse {} as base denomination", s))?
            .try_into()
    }
}

impl std::fmt::Display for DelegationToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base_denom.fmt(f)
    }
}

impl std::fmt::Debug for DelegationToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base_denom.fmt(f)
    }
}

impl PartialEq for DelegationToken {
    fn eq(&self, other: &Self) -> bool {
        self.base_denom.eq(&other.base_denom)
    }
}

impl Eq for DelegationToken {}

impl std::hash::Hash for DelegationToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.base_denom.hash(state)
    }
}

#[cfg(test)]
mod tests {
    use crate::rdsa::{SigningKey, SpendAuth};

    use super::*;

    #[test]
    fn delegation_token_denomination_round_trip() {
        use rand_core::OsRng;

        let ik = IdentityKey(SigningKey::<SpendAuth>::new(OsRng).into());

        let token = DelegationToken::new(ik);

        let denom = token.to_string();
        let token2 = DelegationToken::from_str(&denom).unwrap();
        let denom2 = token2.to_string();

        assert_eq!(denom, denom2);
        assert_eq!(token, token2);
    }
}
