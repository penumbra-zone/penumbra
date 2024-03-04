use anyhow::Result;
use decaf377::Fq;
use decaf377_frost as frost;
use ed25519_consensus::{SigningKey, VerificationKey};
use penumbra_keys::{keys::NullifierKey, FullViewingKey};
use rand_core::CryptoRngCore;
use serde::{Deserialize, Serialize};
use serde_with::{formats::Uppercase, hex::Hex, DisplayFromStr, TryFromInto};
use std::collections::{HashMap, HashSet};

/// A shim to serialize frost::keys::SigningShare
#[serde_as]
#[derive(Serialize, Deserialize)]
struct SigningShareWrapper(#[serde_as(as = "Hex<Uppercase>")] Vec<u8>);

impl From<frost::keys::SigningShare> for SigningShareWrapper {
    fn from(value: frost::keys::SigningShare) -> Self {
        Self(value.serialize())
    }
}

impl TryFrom<SigningShareWrapper> for frost::keys::SigningShare {
    type Error = anyhow::Error;

    fn try_from(value: SigningShareWrapper) -> std::result::Result<Self, Self::Error> {
        Ok(Self::deserialize(value.0)?)
    }
}

/// A shim to serialize frost::keys::VerifyingShare
#[serde_as]
#[derive(Serialize, Deserialize)]
struct VerifyingShareWrapper(#[serde_as(as = "Hex<Uppercase>")] Vec<u8>);

impl From<frost::keys::VerifyingShare> for VerifyingShareWrapper {
    fn from(value: frost::keys::VerifyingShare) -> Self {
        Self(value.serialize())
    }
}

impl TryFrom<VerifyingShareWrapper> for frost::keys::VerifyingShare {
    type Error = anyhow::Error;

    fn try_from(value: VerifyingShareWrapper) -> std::result::Result<Self, Self::Error> {
        Ok(Self::deserialize(value.0)?)
    }
}

/// A shim to serialize SigningKey
#[serde_as]
#[derive(Serialize, Deserialize)]
struct SigningKeyWrapper(#[serde_as(as = "Hex<Uppercase>")] Vec<u8>);

impl From<SigningKey> for SigningKeyWrapper {
    fn from(value: SigningKey) -> Self {
        Self(value.to_bytes().to_vec())
    }
}

impl TryFrom<SigningKeyWrapper> for SigningKey {
    type Error = anyhow::Error;

    fn try_from(value: SigningKeyWrapper) -> std::result::Result<Self, Self::Error> {
        Ok(Self::try_from(value.0.as_slice())?)
    }
}

/// A shim to serialize VerifyingKey
#[serde_as]
#[derive(Serialize, Deserialize)]
struct VerificationKeyWrapper(#[serde_as(as = "Hex<Uppercase>")] Vec<u8>);

impl From<VerificationKey> for VerificationKeyWrapper {
    fn from(value: VerificationKey) -> Self {
        Self(value.to_bytes().to_vec())
    }
}

impl TryFrom<VerificationKeyWrapper> for VerificationKey {
    type Error = anyhow::Error;

    fn try_from(value: VerificationKeyWrapper) -> std::result::Result<Self, Self::Error> {
        Ok(Self::try_from(value.0.as_slice())?)
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    threshold: u16,
    #[serde_as(as = "DisplayFromStr")]
    fvk: FullViewingKey,
    #[serde_as(as = "TryFromInto<SigningShareWrapper>")]
    spend_key_share: frost::keys::SigningShare,
    #[serde_as(as = "TryFromInto<SigningKeyWrapper>")]
    signing_key: SigningKey,
    #[serde_as(
        as = "HashMap<TryFromInto<VerificationKeyWrapper>, TryFromInto<VerifyingShareWrapper>>"
    )]
    verifying_shares: HashMap<VerificationKey, frost::keys::VerifyingShare>,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.threshold == other.threshold
            && self.fvk == other.fvk
            && self.spend_key_share == other.spend_key_share
            // TIMING LEAK
            && self.signing_key.as_bytes() == other.signing_key.as_bytes()
            && self.verifying_shares == other.verifying_shares
    }
}

impl Eq for Config {}

impl Config {
    /// Create a config from the parts that get spit out by the DKG protocol.
    pub(crate) fn from_parts(
        key_package: frost::keys::KeyPackage,
        public_key_package: frost::keys::PublicKeyPackage,
        signing_key: SigningKey,
        verification_keys: Vec<VerificationKey>,
        nullifier_key: Fq,
    ) -> Self {
        let fvk = FullViewingKey::from_components(
            public_key_package
                .group_public()
                .serialize()
                .as_slice()
                .try_into()
                .expect("conversion of a group element to a VerifyingKey should not fail"),
            NullifierKey(nullifier_key),
        );
        let spend_key_share = key_package.secret_share().to_owned();
        let verifying_shares = verification_keys
            .into_iter()
            .map(|vk| {
                let id = frost::Identifier::derive(vk.as_bytes().as_slice())
                    .expect("deriving identifiers should not fail");
                (vk, public_key_package.signer_pubkeys()[&id])
            })
            .collect();
        Self {
            threshold: *key_package.min_signers(),
            fvk,
            spend_key_share,
            signing_key,
            verifying_shares,
        }
    }

    pub fn deal(mut rng: &mut impl CryptoRngCore, t: u16, n: u16) -> Result<Vec<Self>> {
        let signing_keys = (0..n)
            .map(|_| {
                let sk = SigningKey::new(&mut rng);
                let pk = sk.verification_key();
                (pk, sk)
            })
            .collect::<HashMap<_, _>>();
        let identifiers = signing_keys
            .keys()
            .cloned()
            .map(|pk| Ok((pk, frost::Identifier::derive(pk.as_bytes().as_slice())?)))
            .collect::<Result<HashMap<_, _>, frost::Error>>()?;

        let (share_map, public_key_package) = frost::keys::generate_with_dealer(
            n,
            t,
            frost::keys::IdentifierList::Custom(
                identifiers.values().cloned().collect::<Vec<_>>().as_slice(),
            ),
            &mut rng,
        )?;
        let verifying_shares = signing_keys
            .keys()
            .map(|pk| {
                let identifier = frost::Identifier::derive(pk.to_bytes().as_slice())
                    .expect("should be able to derive identifier");
                (pk.clone(), public_key_package.signer_pubkeys()[&identifier])
            })
            .collect::<HashMap<_, _>>();
        // Okay, this conversion is a bit of a hack, but it should work...
        // It's a hack cause we're going via the serialization, but, you know, that should be fine.
        let fvk = FullViewingKey::from_components(
            public_key_package
                .group_public()
                .serialize()
                .as_slice()
                .try_into()
                .expect("conversion of a group element to a VerifyingKey should not fail"),
            NullifierKey(Fq::rand(rng)),
        );

        Ok(signing_keys
            .into_iter()
            .map(|(verification_key, signing_key)| {
                let identifier = identifiers[&verification_key];
                let signing_share = share_map[&identifier].value().clone();
                Self {
                    threshold: t,
                    signing_key,
                    fvk: fvk.clone(),
                    spend_key_share: signing_share,
                    verifying_shares: verifying_shares.clone(),
                }
            })
            .collect())
    }

    pub fn threshold(&self) -> u16 {
        self.threshold
    }

    fn group_public(&self) -> frost::keys::VerifyingKey {
        frost::keys::VerifyingKey::deserialize(
            self.fvk.spend_verification_key().to_bytes().to_vec(),
        )
        .expect("should be able to parse out VerifyingKey from FullViewingKey")
    }

    pub fn key_package(&self) -> frost::keys::KeyPackage {
        let identifier =
            frost::Identifier::derive(&self.signing_key.verification_key().as_bytes().as_slice())
                .expect("deriving our identifier should not fail");

        frost::keys::KeyPackage::new(
            identifier,
            self.spend_key_share,
            self.spend_key_share.into(),
            self.group_public(),
            self.threshold,
        )
    }

    pub fn public_key_package(&self) -> frost::keys::PublicKeyPackage {
        let signer_pubkeys = self
            .verifying_shares
            .iter()
            .map(|(vk, share)| {
                (
                    frost::Identifier::derive(vk.to_bytes().as_slice())
                        .expect("deriving an identifier should not fail"),
                    share.clone(),
                )
            })
            .collect();
        frost::keys::PublicKeyPackage::new(signer_pubkeys, self.group_public())
    }

    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    pub fn fvk(&self) -> &FullViewingKey {
        &self.fvk
    }

    pub fn verification_keys(&self) -> HashSet<VerificationKey> {
        self.verifying_shares.keys().cloned().collect()
    }
}

#[cfg(test)]
mod test {
    use rand_core::OsRng;

    use super::*;

    #[test]
    fn test_config_serialization_roundtrip() -> Result<()> {
        // You can't put 1, because no FUN is allowed
        let config = Config::deal(&mut OsRng, 2, 2)?.pop().unwrap();
        let config_str = serde_json::to_string(&config)?;
        let config2: Config = serde_json::from_str(&config_str)?;
        // Can't derive partial eq, so go field by field
        assert_eq!(config.threshold, config2.threshold);
        assert_eq!(config.fvk, config2.fvk);
        assert_eq!(config.spend_key_share, config2.spend_key_share);
        assert_eq!(config.verifying_shares, config2.verifying_shares);
        Ok(())
    }
}
