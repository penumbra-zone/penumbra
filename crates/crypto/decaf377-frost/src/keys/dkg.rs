//! Distributed key generation without a trusted dealer.
use anyhow::anyhow;
use penumbra_sdk_proto::crypto::decaf377_frost::v1 as pb;

// Copied from frost-ed25519 ("MIT or Apache-2.0") (more or less)

use super::*;

/// DKG Round 1 structures.
pub mod round1 {
    use penumbra_sdk_proto::DomainType;

    use super::*;

    /// The secret package that must be kept in memory by the participant
    /// between the first and second parts of the DKG protocol (round 1).
    ///
    /// # Security
    ///
    /// This package MUST NOT be sent to other participants!
    pub type SecretPackage = frost::keys::dkg::round1::SecretPackage<E>;

    /// The package that must be broadcast by each participant to all other participants
    /// between the first and second parts of the DKG protocol (round 1).
    #[derive(Debug, Clone)]
    pub struct Package(pub(crate) frost::keys::dkg::round1::Package<E>);

    impl From<Package> for pb::DkgRound1Package {
        fn from(value: Package) -> Self {
            Self {
                commitment: Some(pb::VerifiableSecretSharingCommitment {
                    elements: value
                        .0
                        .commitment()
                        .serialize()
                        .into_iter()
                        .map(|x| x.to_vec())
                        .collect(),
                }),
                proof_of_knowledge: value.0.proof_of_knowledge().serialize().to_vec(),
            }
        }
    }

    impl TryFrom<pb::DkgRound1Package> for Package {
        type Error = anyhow::Error;

        fn try_from(value: pb::DkgRound1Package) -> Result<Self, Self::Error> {
            Ok(Self(frost::keys::dkg::round1::Package::new(
                frost::keys::VerifiableSecretSharingCommitment::deserialize(
                    value
                        .commitment
                        .ok_or(anyhow!("DkgRound1Package missing commitment"))?
                        .elements,
                )?,
                frost_core::Signature::deserialize(value.proof_of_knowledge)?,
            )))
        }
    }

    impl DomainType for Package {
        type Proto = pb::DkgRound1Package;
    }
}

/// DKG Round 2 structures.
pub mod round2 {
    use penumbra_sdk_proto::DomainType;

    use super::*;

    /// The secret package that must be kept in memory by the participant
    /// between the second and third parts of the DKG protocol (round 2).
    ///
    /// # Security
    ///
    /// This package MUST NOT be sent to other participants!
    pub type SecretPackage = frost::keys::dkg::round2::SecretPackage<E>;

    /// A package that must be sent by each participant to some other participants
    /// in Round 2 of the DKG protocol. Note that there is one specific package
    /// for each specific recipient, in contrast to Round 1.
    ///
    /// # Security
    ///
    /// The package must be sent on an *confidential* and *authenticated* channel.
    #[derive(Debug, Clone)]
    pub struct Package(pub(crate) frost::keys::dkg::round2::Package<E>);

    impl From<Package> for pb::DkgRound2Package {
        fn from(value: Package) -> Self {
            Self {
                signing_share: Some(pb::SigningShare {
                    scalar: value.0.secret_share().serialize(),
                }),
            }
        }
    }

    impl TryFrom<pb::DkgRound2Package> for Package {
        type Error = anyhow::Error;

        fn try_from(value: pb::DkgRound2Package) -> Result<Self, Self::Error> {
            Ok(Self(frost::keys::dkg::round2::Package::new(
                frost::keys::SigningShare::deserialize(
                    value
                        .signing_share
                        .ok_or(anyhow!("DkgRound2Package missing signing share"))?
                        .scalar,
                )?,
            )))
        }
    }

    impl DomainType for Package {
        type Proto = pb::DkgRound2Package;
    }
}

/// Performs the first part of the distributed key generation protocol
/// for the given participant.
///
/// It returns the [`round1::SecretPackage`] that must be kept in memory
/// by the participant for the other steps, and the [`round1::Package`] that
/// must be sent to other participants.
pub fn part1<R: RngCore + CryptoRng>(
    identifier: Identifier,
    max_signers: u16,
    min_signers: u16,
    mut rng: R,
) -> Result<(round1::SecretPackage, round1::Package), Error> {
    frost::keys::dkg::part1(identifier, max_signers, min_signers, &mut rng)
        .map(|(a, b)| (a, round1::Package(b)))
}

/// Performs the second part of the distributed key generation protocol
/// for the participant holding the given [`round1::SecretPackage`],
/// given the received [`round1::Package`]s received from the other participants.
///
/// It returns the [`round2::SecretPackage`] that must be kept in memory
/// by the participant for the final step, and the [`round2::Package`]s that
/// must be sent to other participants.
pub fn part2(
    secret_package: round1::SecretPackage,
    round1_packages: &HashMap<Identifier, round1::Package>,
) -> Result<(round2::SecretPackage, HashMap<Identifier, round2::Package>), Error> {
    let round1_packages = round1_packages
        .iter()
        .map(|(a, b)| (*a, b.0.clone()))
        .collect();
    frost::keys::dkg::part2(secret_package, &round1_packages).map(|(a, b)| {
        (
            a,
            b.into_iter()
                .map(|(k, v)| (k, round2::Package(v)))
                .collect(),
        )
    })
}
/// Performs the third and final part of the distributed key generation protocol
/// for the participant holding the given [`round2::SecretPackage`],
/// given the received [`round1::Package`]s and [`round2::Package`]s received from
/// the other participants.
///
/// It returns the [`KeyPackage`] that has the long-lived key share for the
/// participant, and the [`PublicKeyPackage`]s that has public information
/// about all participants; both of which are required to compute FROST
/// signatures.
pub fn part3(
    round2_secret_package: &round2::SecretPackage,
    round1_packages: &HashMap<Identifier, round1::Package>,
    round2_packages: &HashMap<Identifier, round2::Package>,
) -> Result<(KeyPackage, PublicKeyPackage), Error> {
    let round1_packages = round1_packages
        .iter()
        .map(|(a, b)| (*a, b.0.clone()))
        .collect();
    let round2_packages = round2_packages
        .iter()
        .map(|(a, b)| (*a, b.0.clone()))
        .collect();
    frost::keys::dkg::part3(round2_secret_package, &round1_packages, &round2_packages)
}
