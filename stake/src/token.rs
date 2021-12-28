use std::str::FromStr;

use penumbra_crypto::asset;
use regex::Regex;
use tendermint::PublicKey;

use crate::VALIDATOR_IDENTITY_BECH32_PREFIX;

/// Delegation tokens represent a share of a particular validator's delegation pool.
pub struct DelegationToken {
    validator_pubkey: PublicKey,
    base_denom: asset::Denom,
}

impl DelegationToken {
    pub fn new(pk: PublicKey) -> Self {
        // This format string needs to be in sync with the asset registry
        let base_denom = asset::REGISTRY
            .parse_denom(&format!(
                "udelegation_{}",
                pk.to_bech32(VALIDATOR_IDENTITY_BECH32_PREFIX)
            ))
            .expect("base denom format is valid");
        DelegationToken {
            validator_pubkey: pk,
            base_denom,
        }
    }

    /// Get the base denomination for this delegation token.
    pub fn base_denom(&self) -> asset::Denom {
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
    pub fn validator(&self) -> PublicKey {
        self.validator_pubkey.clone()
    }
}

impl TryFrom<asset::Denom> for DelegationToken {
    type Error = anyhow::Error;
    fn try_from(value: asset::Denom) -> Result<Self, Self::Error> {
        // Almost all of this code is devoted to parsing Bech32 in the
        // tendermint-specific way, perhaps we could upstream a Bech32 decoding
        // function to tendermint-rs

        let (hrp, data, variant) = bech32::decode(
            // Note: this regex must be in sync with both asset::REGISTRY
            // and VALIDATOR_IDENTITY_BECH32_PREFIX
            Regex::new("udelegation_(?P<data>penumbravalid1[a-zA-HJ-NP-Z0-9]+)")
                .expect("regex is valid")
                .captures(&value.to_string())
                .ok_or_else(|| {
                    anyhow::anyhow!("base denom {} is not a delegation token", value.to_string())
                })?
                .name("data")
                .expect("data is a named capture")
                .as_str(),
        )?;

        if variant != bech32::Variant::Bech32 {
            return Err(anyhow::anyhow!(
                "delegation token denom uses wrong Bech32 variant"
            ));
        }
        if hrp != VALIDATOR_IDENTITY_BECH32_PREFIX {
            return Err(anyhow::anyhow!(
                "wrong Bech32 HRP {}, expected {}",
                hrp,
                VALIDATOR_IDENTITY_BECH32_PREFIX
            ));
        }

        use bech32::FromBase32;
        let data = Vec::<u8>::from_base32(&data).expect("bech32 decoding produces valid base32");

        // some custom tendermint thing (????)
        // see PublicKey::to_bech32 impl
        if &data[0..5] != &[0x16, 0x24, 0xDE, 0x64, 0x20] {
            return Err(anyhow::anyhow!(
                "invalid backward_compatible_amino_prefix bytes"
            ));
        }

        Ok(DelegationToken::new(
            PublicKey::from_raw_ed25519(&data[5..])
                .ok_or_else(|| anyhow::anyhow!("invalid Ed25519 verification key"))?,
        ))
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
    use super::*;

    #[test]
    fn delegation_token_denomination_round_trip() {
        use ed25519_consensus::SigningKey;
        use rand_core::OsRng;

        let vk = PublicKey::Ed25519(SigningKey::new(OsRng).verification_key());

        let token = DelegationToken::new(vk);

        let denom = token.to_string();
        let token2 = DelegationToken::from_str(&denom).unwrap();
        let denom2 = token2.to_string();

        assert_eq!(denom, denom2);
        assert_eq!(token, token2);
    }
}
