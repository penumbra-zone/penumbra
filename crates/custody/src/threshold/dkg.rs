use anyhow::{anyhow, Result};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::Fq;
use decaf377_frost as frost;
use frost::keys::dkg as frost_dkg;
use std::collections::{HashMap, HashSet};
mod encryption;
use ed25519_consensus::{Signature, SigningKey, VerificationKey};
use encryption::EncryptionKey;
use penumbra_proto::{custody::threshold::v1 as pb, DomainType, Message};
use rand_core::CryptoRngCore;

use self::encryption::DecryptionKey;

use super::Config;

/// A commitment to our share of the nullifier.
///
/// In order to check a commitment, simply use the equality operator, after recreating
/// the commitment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NullifierCommitment([u8; 32]);

impl NullifierCommitment {
    /// Create a new commitment to a given nullifier
    fn create(share: Fq) -> Self {
        let mut state = blake2b_simd::Params::new()
            .personal(b"dkg-commit")
            .to_state();
        share
            .serialize_compressed(&mut state)
            .expect("failed to serialize Fq element");
        let out = state.finalize().as_array()[..32]
            .try_into()
            .expect("array conversion should not fail");
        Self(out)
    }

    fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl TryFrom<&[u8]> for NullifierCommitment {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let bytes: [u8; 32] = value.try_into()?;
        Ok(Self(bytes))
    }
}

/// The message we send in round 1 of the DKG protocol
#[derive(Clone)]
pub struct Round1 {
    /// The underlying FROST package.
    package: frost_dkg::round1::Package,
    /// A commitment to our share of the nullifier.
    nullifier_commitment: NullifierCommitment,
    /// An encryption key to receive the encrypted packages in round 2.
    epk: EncryptionKey,
    /// The verification key that will act as our identity from now on.
    vk: VerificationKey,
}

impl From<Round1> for pb::DkgRound1 {
    fn from(value: Round1) -> Self {
        Self {
            pkg: Some(value.package.into()),
            nullifier_commitment: value.nullifier_commitment.as_bytes().to_vec(),
            epk: value.epk.as_bytes().to_vec(),
            vk: value.vk.as_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::DkgRound1> for Round1 {
    type Error = anyhow::Error;

    fn try_from(value: pb::DkgRound1) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            package: value
                .pkg
                .ok_or(anyhow!("DkgRound1 missing pkg"))?
                .try_into()?,
            nullifier_commitment: value.nullifier_commitment.as_slice().try_into()?,
            epk: value.epk.as_slice().try_into()?,
            vk: value.vk.as_slice().try_into()?,
        })
    }
}

impl DomainType for Round1 {
    type Proto = pb::DkgRound1;
}

fn round2_inner_to_pb(
    encrypted_packages: HashMap<VerificationKey, Vec<u8>>,
    nullifier: Fq,
) -> pb::dkg_round2::Inner {
    let nullifier = {
        let mut bytes = Vec::new();
        nullifier
            .serialize_compressed(&mut bytes)
            .expect("field serialization should not fail");
        bytes
    };
    // Need to sort to guarantee a deterministic encoding for signing.
    let encrypted_packages = {
        let mut acc: Vec<_> = encrypted_packages
            .into_iter()
            .map(|(k, v)| pb::dkg_round2::TargetedPackage {
                vk: k.as_bytes().to_vec(),
                encrypted_package: v,
            })
            .collect();
        acc.sort_by_key(|x| x.vk.clone());
        acc
    };
    pb::dkg_round2::Inner {
        encrypted_packages,
        nullifier,
    }
}

/// The message we send in round 2 of the DKG protocol.
#[derive(Clone, Debug)]
pub struct Round2 {
    /// For each other participant, a ciphertext containing the FROST package they need.
    encrypted_packages: HashMap<VerificationKey, Vec<u8>>,
    /// Our revealed nullifier (share) from the previous round.
    nullifier: Fq,
    /// A declaration of our identity.
    vk: VerificationKey,
    // A signature over the encrypted packages and nullifier.
    sig: Signature,
}

impl From<Round2> for pb::DkgRound2 {
    fn from(value: Round2) -> Self {
        Self {
            inner: Some(round2_inner_to_pb(
                value.encrypted_packages,
                value.nullifier,
            )),
            vk: value.vk.as_bytes().to_vec(),
            sig: value.sig.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::DkgRound2> for Round2 {
    type Error = anyhow::Error;

    fn try_from(value: pb::DkgRound2) -> std::result::Result<Self, Self::Error> {
        let inner = value.inner.ok_or(anyhow!("DkgRound2 missing inner"))?;
        Ok(Self {
            encrypted_packages: inner
                .encrypted_packages
                .into_iter()
                .map(|x| Ok((x.vk.as_slice().try_into()?, x.encrypted_package)))
                .collect::<Result<HashMap<_, _>, Self::Error>>()?,
            nullifier: Fq::deserialize_compressed(inner.nullifier.as_slice())?,
            vk: value.vk.as_slice().try_into()?,
            sig: value.sig.as_slice().try_into()?,
        })
    }
}

impl DomainType for Round2 {
    type Proto = pb::DkgRound2;
}

impl Round2 {
    fn make(
        sk: &SigningKey,
        encrypted_packages: HashMap<VerificationKey, Vec<u8>>,
        nullifier: Fq,
    ) -> Self {
        let data = round2_inner_to_pb(encrypted_packages.clone(), nullifier).encode_to_vec();
        let sig = sk.sign(&data);
        Self {
            encrypted_packages,
            nullifier,
            vk: sk.verification_key(),
            sig,
        }
    }

    fn encrypted_packages(self) -> Result<(VerificationKey, HashMap<VerificationKey, Vec<u8>>)> {
        let data =
            round2_inner_to_pb(self.encrypted_packages.clone(), self.nullifier).encode_to_vec();
        self.vk.verify(&self.sig, &data)?;
        Ok((self.vk, self.encrypted_packages))
    }
}

/// The state we need to remember after round 1.
pub struct Round1State {
    /// This is what FROST tells us to remember.
    secret_package: frost_dkg::round1::SecretPackage,
    /// We remember our nullifier share, so we can open it later.
    nullifier: Fq,
    /// We remember our signing key, for the final config, and to sign the next round.
    sk: SigningKey,
    /// We remember the new decryption key we've created, to decrypt the next round's packages.
    edk: DecryptionKey,
}

/// The state we need to remember after round 2.
pub struct Round2State {
    /// This is what FROST tells us to remember.
    secret_package: frost_dkg::round2::SecretPackage,
    /// FROST round 3 will need this data.
    round1_packages: HashMap<frost::Identifier, frost_dkg::round1::Package>,
    /// We want to keep a list of verification keys, and we need to remember the commitments
    /// so that we can check the openings in the next round anyways.
    ///
    /// Caching the identifier mappings here is useful as well.
    associated_data: HashMap<VerificationKey, (frost::Identifier, NullifierCommitment)>,
    /// Our share of the nullifier, which we need to sum with the opened nullifiers next round.
    nullifier: Fq,
    /// We keep the signing key to save in the config
    sk: SigningKey,
    /// We keep this, so that we can decrypt the packages.
    edk: DecryptionKey,
}

pub fn round1(mut rng: impl CryptoRngCore, t: u16, n: u16) -> Result<(Round1, Round1State)> {
    // hack to get around SigningKey taking rng by value
    let sk = SigningKey::new(&mut rng);
    let vk = sk.verification_key();
    let id = frost::Identifier::derive(vk.as_bytes())?;
    let (secret_package, package) = frost_dkg::part1(id, n, t, &mut rng)?;
    let edk = DecryptionKey::new(&mut rng);
    let epk = edk.public();
    let nullifier = Fq::rand(&mut rng);
    let nullifier_commitment = NullifierCommitment::create(nullifier);
    let round1 = Round1 {
        package,
        nullifier_commitment,
        epk,
        vk,
    };
    let state = Round1State {
        secret_package,
        nullifier,
        sk,
        edk,
    };
    Ok((round1, state))
}

pub fn round2(
    mut rng: impl CryptoRngCore,
    state: Round1State,
    messages: Vec<Round1>,
) -> Result<(Round2, Round2State)> {
    // Check that all verification keys are unique, and not equal to my own
    {
        let mut seen = HashSet::new();
        seen.insert(state.sk.verification_key());
        for m in &messages {
            if seen.contains(&m.vk) {
                anyhow::bail!("duplicate verification key in messages");
            }
        }
    }

    let associated_info = messages
        .into_iter()
        .map(|x| {
            Ok((
                x.vk,
                (
                    frost::Identifier::derive(x.vk.as_bytes())?,
                    x.epk,
                    x.package,
                    x.nullifier_commitment,
                ),
            ))
        })
        .collect::<Result<HashMap<_, _>>>()?;
    let round1_packages = associated_info
        .iter()
        .map(|(_, (id, _, package, _))| (*id, package.clone()))
        .collect();
    let (secret_package, output_packages) =
        frost_dkg::part2(state.secret_package, &round1_packages)?;
    let round2 = {
        let encrypted_packages = associated_info
            .iter()
            .map(|(vk, (id, epk, _, _))| {
                let package = output_packages
                    .get(id)
                    .ok_or(anyhow!("unknown identifier: {:?}", id))?;
                let ciphertext = epk.encrypt(&mut rng, &package.encode_to_vec());
                Ok((vk.clone(), ciphertext))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        Round2::make(&state.sk, encrypted_packages, state.nullifier)
    };
    let state = Round2State {
        secret_package,
        round1_packages,
        associated_data: associated_info
            .into_iter()
            .map(|(vk, (id, _, _, com))| (vk, (id, com)))
            .collect(),
        nullifier: state.nullifier,
        sk: state.sk,
        edk: state.edk,
    };
    Ok((round2, state))
}

pub fn round3(
    mut rng: impl CryptoRngCore,
    state: Round2State,
    messages: Vec<Round2>,
) -> Result<Config> {
    let nullifier_key = {
        let mut acc = state.nullifier;
        for message in &messages {
            if !state.associated_data.contains_key(&message.vk) {
                anyhow::bail!("unknown verification key in round 2 message");
            }
            if NullifierCommitment::create(message.nullifier)
                != state.associated_data[&message.vk].1
            {
                anyhow::bail!("opened nullifier did not match commitment");
            }
            acc += message.nullifier;
        }
        acc
    };
    let round2_packages = messages
        .into_iter()
        .map(|x| {
            let (vk, encrypted_packages) = x.encrypted_packages()?;
            let my_ciphertext = encrypted_packages
                .get(&state.sk.verification_key())
                .ok_or(anyhow!("no encrypted package for this recipient"))?;
            let my_plaintext = state.edk.decrypt(&mut rng, &my_ciphertext)?;
            let package = frost_dkg::round2::Package::decode(my_plaintext.as_slice())?;
            let id = state
                .associated_data
                .get(&vk)
                .ok_or(anyhow!("unknown verification key in round 2 message"))?
                .0;
            Ok((id, package))
        })
        .collect::<Result<HashMap<_, _>>>()?;
    let (key_package, public_key_package) = frost_dkg::part3(
        &state.secret_package,
        &state.round1_packages,
        &round2_packages,
    )?;
    let verification_keys = {
        let mut acc: Vec<_> = state.associated_data.keys().cloned().collect();
        acc.push(state.sk.verification_key());
        acc
    };
    Ok(Config::from_parts(
        key_package,
        public_key_package,
        state.sk,
        verification_keys,
        nullifier_key,
    ))
}
