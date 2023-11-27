use anyhow::Result;
use ark_ff::UniformRand;
use decaf377::Fq;
use decaf377_frost as frost;
use ed25519_consensus::{SigningKey, VerificationKey};
use penumbra_keys::{keys::NullifierKey, FullViewingKey};
use rand_core::CryptoRngCore;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Config {
    pub key_package: frost::keys::KeyPackage,
    pub public_key_package: frost::keys::PublicKeyPackage,
    pub signing_key: SigningKey,
    pub fvk: FullViewingKey,
    pub verification_keys: HashSet<VerificationKey>,
}

impl Config {
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
        let verification_keys = signing_keys.keys().cloned().collect::<HashSet<_>>();
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
                let key_package = frost::keys::KeyPackage::new(
                    identifier,
                    signing_share,
                    signing_share.into(),
                    public_key_package.group_public().clone(),
                    t,
                );
                Self {
                    key_package,
                    public_key_package: public_key_package.clone(),
                    signing_key,
                    fvk: fvk.clone(),
                    verification_keys: verification_keys.clone(),
                }
            })
            .collect())
    }
}
