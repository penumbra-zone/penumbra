//! A module for grouping several setup ceremonies into a single one.
//!
//! This also matches the coordination strategy we have for phase2,
//! along with the corresponding protobufs.
use std::array;

use crate::single::{
    self, circuit_degree, group::F, log::ContributionHash, DLogProof, ExtraTransitionInformation,
    LinkingProof, Phase1CRSElements, Phase1Contribution, Phase1RawCRSElements,
    Phase1RawContribution, Phase2CRSElements, Phase2Contribution, Phase2RawCRSElements,
    Phase2RawContribution,
};
use anyhow::{anyhow, Result};
use ark_groth16::ProvingKey;
use ark_relations::r1cs::ConstraintMatrices;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::Bls12_377;
use penumbra_dex::{swap::proof::SwapCircuit, swap_claim::proof::SwapClaimCircuit};
use penumbra_governance::DelegatorVoteCircuit;
use penumbra_proof_params::generate_constraint_matrices;
use penumbra_proto::tools::summoning::v1alpha1::{self as pb};
use penumbra_shielded_pool::{NullifierDerivationCircuit, OutputCircuit, SpendCircuit};
use penumbra_stake::UndelegateClaimCircuit;

use rand_core::CryptoRngCore;

// Some helper functions since we have to use these seventeen billion times

fn to_bytes<T: CanonicalSerialize>(t: &T) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    t.serialize_uncompressed(&mut out)?;
    Ok(out)
}

fn from_bytes<T: CanonicalDeserialize>(data: &[u8]) -> Result<T> {
    Ok(T::deserialize_uncompressed(data)?)
}

fn from_bytes_unchecked<T: CanonicalDeserialize>(data: &[u8]) -> Result<T> {
    Ok(T::deserialize_uncompressed_unchecked(data)?)
}

pub const NUM_CIRCUITS: usize = 7;

/// Generate all of the circuits as matrices.
fn circuits() -> [ConstraintMatrices<F>; NUM_CIRCUITS] {
    [
        generate_constraint_matrices::<SpendCircuit>(),
        generate_constraint_matrices::<OutputCircuit>(),
        generate_constraint_matrices::<DelegatorVoteCircuit>(),
        generate_constraint_matrices::<UndelegateClaimCircuit>(),
        generate_constraint_matrices::<SwapCircuit>(),
        generate_constraint_matrices::<SwapClaimCircuit>(),
        generate_constraint_matrices::<NullifierDerivationCircuit>(),
    ]
}

/// Holds all of the CRS elements for phase2 in one struct, before validation.
#[derive(Clone, Debug)]
pub struct Phase2RawCeremonyCRS([Phase2RawCRSElements; NUM_CIRCUITS]);

impl Phase2RawCeremonyCRS {
    /// Skip validation, performing the conversion anyways.
    ///
    /// Useful when parsing known good data.
    pub fn assume_valid(self) -> Phase2CeremonyCRS {
        match self.0 {
            [x0, x1, x2, x3, x4, x5, x6] => Phase2CeremonyCRS([
                x0.assume_valid(),
                x1.assume_valid(),
                x2.assume_valid(),
                x3.assume_valid(),
                x4.assume_valid(),
                x5.assume_valid(),
                x6.assume_valid(),
            ]),
        }
    }
}

impl TryInto<pb::CeremonyCrs> for Phase2RawCeremonyCRS {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<pb::CeremonyCrs> {
        Ok(pb::CeremonyCrs {
            spend: to_bytes(&self.0[0])?,
            output: to_bytes(&self.0[1])?,
            delegator_vote: to_bytes(&self.0[2])?,
            undelegate_claim: to_bytes(&self.0[3])?,
            swap: to_bytes(&self.0[4])?,
            swap_claim: to_bytes(&self.0[5])?,
            nullifer_derivation_crs: to_bytes(&self.0[6])?,
        })
    }
}

impl TryFrom<pb::CeremonyCrs> for Phase2RawCeremonyCRS {
    type Error = anyhow::Error;

    fn try_from(value: pb::CeremonyCrs) -> std::result::Result<Self, Self::Error> {
        Ok(Self([
            from_bytes::<Phase2RawCRSElements>(value.spend.as_slice())?,
            from_bytes::<Phase2RawCRSElements>(value.output.as_slice())?,
            from_bytes::<Phase2RawCRSElements>(value.delegator_vote.as_slice())?,
            from_bytes::<Phase2RawCRSElements>(value.undelegate_claim.as_slice())?,
            from_bytes::<Phase2RawCRSElements>(value.swap.as_slice())?,
            from_bytes::<Phase2RawCRSElements>(value.swap_claim.as_slice())?,
            from_bytes::<Phase2RawCRSElements>(value.nullifer_derivation_crs.as_slice())?,
        ]))
    }
}

/// Holds all of the CRS elements for phase2 in one struct.
#[derive(Clone, Debug)]
pub struct Phase2CeremonyCRS([Phase2CRSElements; NUM_CIRCUITS]);

impl From<Phase2CeremonyCRS> for Phase2RawCeremonyCRS {
    fn from(value: Phase2CeremonyCRS) -> Self {
        Self(array::from_fn(|i| value.0[i].raw.clone()))
    }
}

impl TryFrom<Phase2CeremonyCRS> for pb::CeremonyCrs {
    type Error = anyhow::Error;

    fn try_from(data: Phase2CeremonyCRS) -> Result<pb::CeremonyCrs> {
        Phase2RawCeremonyCRS::from(data).try_into()
    }
}

impl Phase2CeremonyCRS {
    pub fn root() -> Result<Self> {
        let [c0, c1, c2, c3, c4, c5, c6] = circuits();
        Ok(Self([
            Phase2CRSElements::dummy_root(circuit_degree(&c0)?),
            Phase2CRSElements::dummy_root(circuit_degree(&c1)?),
            Phase2CRSElements::dummy_root(circuit_degree(&c2)?),
            Phase2CRSElements::dummy_root(circuit_degree(&c3)?),
            Phase2CRSElements::dummy_root(circuit_degree(&c4)?),
            Phase2CRSElements::dummy_root(circuit_degree(&c5)?),
            Phase2CRSElements::dummy_root(circuit_degree(&c6)?),
        ]))
    }
}

/// All phase2 contributions, before they've been validated.
#[derive(Clone, Debug)]
pub struct Phase2RawCeremonyContribution([Phase2RawContribution; NUM_CIRCUITS]);

impl TryInto<pb::participate_request::Contribution> for Phase2RawCeremonyContribution {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<pb::participate_request::Contribution> {
        Ok(pb::participate_request::Contribution {
            updated: Some(pb::CeremonyCrs {
                spend: to_bytes(&self.0[0].new_elements)?,
                output: to_bytes(&self.0[1].new_elements)?,
                delegator_vote: to_bytes(&self.0[2].new_elements)?,
                undelegate_claim: to_bytes(&self.0[3].new_elements)?,
                swap: to_bytes(&self.0[4].new_elements)?,
                swap_claim: to_bytes(&self.0[5].new_elements)?,
                nullifer_derivation_crs: to_bytes(&self.0[6].new_elements)?,
            }),
            update_proofs: Some(pb::CeremonyLinkingProof {
                spend: to_bytes(&self.0[0].linking_proof)?,
                output: to_bytes(&self.0[1].linking_proof)?,
                delegator_vote: to_bytes(&self.0[2].linking_proof)?,
                undelegate_claim: to_bytes(&self.0[3].linking_proof)?,
                swap: to_bytes(&self.0[4].linking_proof)?,
                swap_claim: to_bytes(&self.0[5].linking_proof)?,
                nullifer_derivation_crs: to_bytes(&self.0[6].linking_proof)?,
            }),
            parent_hashes: Some(pb::CeremonyParentHashes {
                spend: self.0[0].parent.0.to_vec(),
                output: self.0[1].parent.0.to_vec(),
                delegator_vote: self.0[2].parent.0.to_vec(),
                undelegate_claim: self.0[3].parent.0.to_vec(),
                swap: self.0[4].parent.0.to_vec(),
                swap_claim: self.0[5].parent.0.to_vec(),
                nullifer_derivation_crs: self.0[6].parent.0.to_vec(),
            }),
        })
    }
}

impl TryFrom<pb::participate_request::Contribution> for Phase2RawCeremonyContribution {
    type Error = anyhow::Error;

    fn try_from(value: pb::participate_request::Contribution) -> Result<Self> {
        Ok(Self([
            Phase2RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .spend
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase2RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .spend
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<DLogProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .spend
                        .as_slice(),
                )?,
            },
            Phase2RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .output
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase2RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .output
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<DLogProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .output
                        .as_slice(),
                )?,
            },
            Phase2RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .delegator_vote
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase2RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .delegator_vote
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<DLogProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .delegator_vote
                        .as_slice(),
                )?,
            },
            Phase2RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .undelegate_claim
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase2RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .undelegate_claim
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<DLogProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .undelegate_claim
                        .as_slice(),
                )?,
            },
            Phase2RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .swap
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase2RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .swap
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<DLogProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .swap
                        .as_slice(),
                )?,
            },
            Phase2RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .swap_claim
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase2RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .swap_claim
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<DLogProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .swap_claim
                        .as_slice(),
                )?,
            },
            Phase2RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .nullifer_derivation_crs
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase2RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .nullifer_derivation_crs
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<DLogProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .nullifer_derivation_crs
                        .as_slice(),
                )?,
            },
        ]))
    }
}

impl Phase2RawCeremonyContribution {
    /// Validate that this contribution is internally consistent.
    ///
    /// This doesn't check that it's connected to the right parent though, which is an additional
    /// step you want to do.
    pub fn validate(
        self,
        rng: &mut impl CryptoRngCore,
        root: &Phase2CeremonyCRS,
    ) -> Option<Phase2CeremonyContribution> {
        // Not happy at the need to copy here, but this avoids the need for a default impl or
        // unsafe.
        for (x, root_i) in self.0.iter().cloned().zip(root.0.iter()) {
            x.validate(rng, root_i)?;
        }
        Some(self.assume_valid())
    }

    /// Skip validation, performing the conversion anyways.
    ///
    /// Useful when parsing known good data.
    pub fn assume_valid(self) -> Phase2CeremonyContribution {
        // This avoids a copy, and will break if we change the size:
        match self.0 {
            [x0, x1, x2, x3, x4, x5, x6] => Phase2CeremonyContribution([
                x0.assume_valid(),
                x1.assume_valid(),
                x2.assume_valid(),
                x3.assume_valid(),
                x4.assume_valid(),
                x5.assume_valid(),
                x6.assume_valid(),
            ]),
        }
    }
}

/// Holds all of the phase2 contributions in a single package.
#[derive(Clone, Debug)]
pub struct Phase2CeremonyContribution([Phase2Contribution; NUM_CIRCUITS]);

impl From<Phase2CeremonyContribution> for Phase2RawCeremonyContribution {
    fn from(value: Phase2CeremonyContribution) -> Self {
        let out: [Phase2RawContribution; NUM_CIRCUITS] =
            array::from_fn(|i| Phase2RawContribution::from(value.0[i].clone()));
        Self(out)
    }
}

impl TryFrom<Phase2CeremonyContribution> for pb::participate_request::Contribution {
    type Error = anyhow::Error;

    fn try_from(data: Phase2CeremonyContribution) -> Result<pb::participate_request::Contribution> {
        Phase2RawCeremonyContribution::from(data).try_into()
    }
}

impl Phase2CeremonyContribution {
    /// Get the new elements contained in this contribution
    pub fn new_elements(&self) -> Phase2CeremonyCRS {
        Phase2CeremonyCRS(array::from_fn(|i| self.0[i].new_elements.clone()))
    }

    /// Check that this contribution is linked to some specific parent elements.
    #[must_use]
    pub fn is_linked_to(&self, parent: &Phase2CeremonyCRS) -> bool {
        self.0
            .iter()
            .zip(parent.0.iter())
            .all(|(x, y)| x.is_linked_to(y))
    }

    pub fn make<R: CryptoRngCore>(rng: &mut R, old: &Phase2CeremonyCRS) -> Self {
        Self(array::from_fn(|i| {
            Phase2Contribution::make(rng, ContributionHash::dummy(), &old.0.as_ref()[i])
        }))
    }
}

// TODO: Make the phase 1 and phase 2 functionality generic

/// Holds all of the CRS elements for phase1 in one struct, before validation.
#[derive(Clone, Debug)]
pub struct Phase1RawCeremonyCRS([Phase1RawCRSElements; NUM_CIRCUITS]);

impl Phase1RawCeremonyCRS {
    /// Skip validation, performing the conversion anyways.
    ///
    /// Useful when parsing known good data.
    pub fn assume_valid(self) -> Phase1CeremonyCRS {
        match self.0 {
            [x0, x1, x2, x3, x4, x5, x6] => Phase1CeremonyCRS([
                x0.assume_valid(),
                x1.assume_valid(),
                x2.assume_valid(),
                x3.assume_valid(),
                x4.assume_valid(),
                x5.assume_valid(),
                x6.assume_valid(),
            ]),
        }
    }

    /// This should only be used when the data is known to be from a trusted source.
    pub fn unchecked_from_protobuf(value: pb::CeremonyCrs) -> anyhow::Result<Self> {
        Ok(Self([
            from_bytes_unchecked::<Phase1RawCRSElements>(value.spend.as_slice())?,
            from_bytes_unchecked::<Phase1RawCRSElements>(value.output.as_slice())?,
            from_bytes_unchecked::<Phase1RawCRSElements>(value.delegator_vote.as_slice())?,
            from_bytes_unchecked::<Phase1RawCRSElements>(value.undelegate_claim.as_slice())?,
            from_bytes_unchecked::<Phase1RawCRSElements>(value.swap.as_slice())?,
            from_bytes_unchecked::<Phase1RawCRSElements>(value.swap_claim.as_slice())?,
            from_bytes_unchecked::<Phase1RawCRSElements>(value.nullifer_derivation_crs.as_slice())?,
        ]))
    }
}

impl TryInto<pb::CeremonyCrs> for Phase1RawCeremonyCRS {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<pb::CeremonyCrs> {
        Ok(pb::CeremonyCrs {
            spend: to_bytes(&self.0[0])?,
            output: to_bytes(&self.0[1])?,
            delegator_vote: to_bytes(&self.0[2])?,
            undelegate_claim: to_bytes(&self.0[3])?,
            swap: to_bytes(&self.0[4])?,
            swap_claim: to_bytes(&self.0[5])?,
            nullifer_derivation_crs: to_bytes(&self.0[6])?,
        })
    }
}

impl TryFrom<pb::CeremonyCrs> for Phase1RawCeremonyCRS {
    type Error = anyhow::Error;

    fn try_from(value: pb::CeremonyCrs) -> std::result::Result<Self, Self::Error> {
        Ok(Self([
            from_bytes::<Phase1RawCRSElements>(value.spend.as_slice())?,
            from_bytes::<Phase1RawCRSElements>(value.output.as_slice())?,
            from_bytes::<Phase1RawCRSElements>(value.delegator_vote.as_slice())?,
            from_bytes::<Phase1RawCRSElements>(value.undelegate_claim.as_slice())?,
            from_bytes::<Phase1RawCRSElements>(value.swap.as_slice())?,
            from_bytes::<Phase1RawCRSElements>(value.swap_claim.as_slice())?,
            from_bytes::<Phase1RawCRSElements>(value.nullifer_derivation_crs.as_slice())?,
        ]))
    }
}

/// Holds all of the CRS elements for phase1 in one struct.
#[derive(Clone, Debug, PartialEq)]
pub struct Phase1CeremonyCRS([Phase1CRSElements; NUM_CIRCUITS]);

impl From<Phase1CeremonyCRS> for Phase1RawCeremonyCRS {
    fn from(value: Phase1CeremonyCRS) -> Self {
        Self(array::from_fn(|i| value.0[i].raw.clone()))
    }
}

impl TryFrom<Phase1CeremonyCRS> for pb::CeremonyCrs {
    type Error = anyhow::Error;

    fn try_from(data: Phase1CeremonyCRS) -> Result<pb::CeremonyCrs> {
        Phase1RawCeremonyCRS::from(data).try_into()
    }
}

impl Phase1CeremonyCRS {
    pub fn root() -> Result<Self> {
        let [c0, c1, c2, c3, c4, c5, c6] = circuits();
        Ok(Self([
            Phase1CRSElements::root(circuit_degree(&c0)?),
            Phase1CRSElements::root(circuit_degree(&c1)?),
            Phase1CRSElements::root(circuit_degree(&c2)?),
            Phase1CRSElements::root(circuit_degree(&c3)?),
            Phase1CRSElements::root(circuit_degree(&c4)?),
            Phase1CRSElements::root(circuit_degree(&c5)?),
            Phase1CRSElements::root(circuit_degree(&c6)?),
        ]))
    }
}

/// All phase1 contributions, before they've been validated.
#[derive(Clone, Debug)]
pub struct Phase1RawCeremonyContribution([Phase1RawContribution; NUM_CIRCUITS]);

impl TryInto<pb::participate_request::Contribution> for Phase1RawCeremonyContribution {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<pb::participate_request::Contribution> {
        Ok(pb::participate_request::Contribution {
            updated: Some(pb::CeremonyCrs {
                spend: to_bytes(&self.0[0].new_elements)?,
                output: to_bytes(&self.0[1].new_elements)?,
                delegator_vote: to_bytes(&self.0[2].new_elements)?,
                undelegate_claim: to_bytes(&self.0[3].new_elements)?,
                swap: to_bytes(&self.0[4].new_elements)?,
                swap_claim: to_bytes(&self.0[5].new_elements)?,
                nullifer_derivation_crs: to_bytes(&self.0[6].new_elements)?,
            }),
            update_proofs: Some(pb::CeremonyLinkingProof {
                spend: to_bytes(&self.0[0].linking_proof)?,
                output: to_bytes(&self.0[1].linking_proof)?,
                delegator_vote: to_bytes(&self.0[2].linking_proof)?,
                undelegate_claim: to_bytes(&self.0[3].linking_proof)?,
                swap: to_bytes(&self.0[4].linking_proof)?,
                swap_claim: to_bytes(&self.0[5].linking_proof)?,
                nullifer_derivation_crs: to_bytes(&self.0[6].linking_proof)?,
            }),
            parent_hashes: Some(pb::CeremonyParentHashes {
                spend: self.0[0].parent.0.to_vec(),
                output: self.0[1].parent.0.to_vec(),
                delegator_vote: self.0[2].parent.0.to_vec(),
                undelegate_claim: self.0[3].parent.0.to_vec(),
                swap: self.0[4].parent.0.to_vec(),
                swap_claim: self.0[5].parent.0.to_vec(),
                nullifer_derivation_crs: self.0[6].parent.0.to_vec(),
            }),
        })
    }
}

impl TryFrom<pb::participate_request::Contribution> for Phase1RawCeremonyContribution {
    type Error = anyhow::Error;

    fn try_from(value: pb::participate_request::Contribution) -> Result<Self> {
        Ok(Self([
            Phase1RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .spend
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase1RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .spend
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<LinkingProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .spend
                        .as_slice(),
                )?,
            },
            Phase1RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .output
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase1RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .output
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<LinkingProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .output
                        .as_slice(),
                )?,
            },
            Phase1RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .delegator_vote
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase1RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .delegator_vote
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<LinkingProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .delegator_vote
                        .as_slice(),
                )?,
            },
            Phase1RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .undelegate_claim
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase1RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .undelegate_claim
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<LinkingProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .undelegate_claim
                        .as_slice(),
                )?,
            },
            Phase1RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .swap
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase1RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .swap
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<LinkingProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .swap
                        .as_slice(),
                )?,
            },
            Phase1RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .swap_claim
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase1RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .swap_claim
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<LinkingProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .swap_claim
                        .as_slice(),
                )?,
            },
            Phase1RawContribution {
                parent: ContributionHash::try_from(
                    value
                        .parent_hashes
                        .as_ref()
                        .ok_or(anyhow!("no parent hashes"))?
                        .nullifer_derivation_crs
                        .as_slice(),
                )?,
                new_elements: from_bytes::<Phase1RawCRSElements>(
                    value
                        .updated
                        .as_ref()
                        .ok_or(anyhow!("no updated"))?
                        .nullifer_derivation_crs
                        .as_slice(),
                )?,
                linking_proof: from_bytes::<LinkingProof>(
                    value
                        .update_proofs
                        .as_ref()
                        .ok_or(anyhow!("no update proofs"))?
                        .nullifer_derivation_crs
                        .as_slice(),
                )?,
            },
        ]))
    }
}

impl Phase1RawCeremonyContribution {
    /// Validate that this contribution is internally consistent.
    ///
    /// This doesn't check that it's connected to the right parent though, which is an additional
    /// step you want to do.
    pub fn validate(self, rng: &mut impl CryptoRngCore) -> Option<Phase1CeremonyContribution> {
        // Not happy at the need to copy here, but this avoids the need for a default impl or
        // unsafe.
        for x in self.0.iter().cloned() {
            x.validate(rng)?;
        }
        Some(self.assume_valid())
    }

    /// Skip validation, performing the conversion anyways.
    ///
    /// Useful when parsing known good data.
    pub fn assume_valid(self) -> Phase1CeremonyContribution {
        // This avoids a copy, and will break if we change the size:
        match self.0 {
            [x0, x1, x2, x3, x4, x5, x6] => Phase1CeremonyContribution([
                x0.assume_valid(),
                x1.assume_valid(),
                x2.assume_valid(),
                x3.assume_valid(),
                x4.assume_valid(),
                x5.assume_valid(),
                x6.assume_valid(),
            ]),
        }
    }
}

/// Holds all of the phase1 contributions in a single package.
#[derive(Clone, Debug)]
pub struct Phase1CeremonyContribution([Phase1Contribution; NUM_CIRCUITS]);

impl From<Phase1CeremonyContribution> for Phase1RawCeremonyContribution {
    fn from(value: Phase1CeremonyContribution) -> Self {
        let out: [Phase1RawContribution; NUM_CIRCUITS] =
            array::from_fn(|i| Phase1RawContribution::from(value.0[i].clone()));
        Self(out)
    }
}

impl TryFrom<Phase1CeremonyContribution> for pb::participate_request::Contribution {
    type Error = anyhow::Error;

    fn try_from(data: Phase1CeremonyContribution) -> Result<pb::participate_request::Contribution> {
        Phase1RawCeremonyContribution::from(data).try_into()
    }
}

impl Phase1CeremonyContribution {
    /// Get the new elements contained in this contribution
    pub fn new_elements(&self) -> Phase1CeremonyCRS {
        Phase1CeremonyCRS(array::from_fn(|i| self.0[i].new_elements.clone()))
    }

    /// Check that this contribution is linked to some specific parent elements.
    #[must_use]
    pub fn is_linked_to(&self, parent: &Phase1CeremonyCRS) -> bool {
        self.0
            .iter()
            .zip(parent.0.iter())
            .all(|(x, y)| x.is_linked_to(y))
    }

    pub fn make<R: CryptoRngCore>(rng: &mut R, old: &Phase1CeremonyCRS) -> Self {
        Self(array::from_fn(|i| {
            Phase1Contribution::make(rng, ContributionHash::dummy(), &old.0.as_ref()[i])
        }))
    }
}

#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct AllExtraTransitionInformation([ExtraTransitionInformation; NUM_CIRCUITS]);

impl AllExtraTransitionInformation {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        to_bytes(self)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(from_bytes_unchecked::<Self>(data)?)
    }
}

/// Transition between phase1 and phase2, producing extra information to be saved.
pub fn transition(
    phase1: &Phase1CeremonyCRS,
) -> Result<(AllExtraTransitionInformation, Phase2CeremonyCRS)> {
    let [c0, c1, c2, c3, c4, c5, c6] = circuits();

    let (e0, p0) = single::transition(&phase1.0[0], &c0)?;
    let (e1, p1) = single::transition(&phase1.0[1], &c1)?;
    let (e2, p2) = single::transition(&phase1.0[2], &c2)?;
    let (e3, p3) = single::transition(&phase1.0[3], &c3)?;
    let (e4, p4) = single::transition(&phase1.0[4], &c4)?;
    let (e5, p5) = single::transition(&phase1.0[5], &c5)?;
    let (e6, p6) = single::transition(&phase1.0[6], &c6)?;

    Ok((
        AllExtraTransitionInformation([e0, e1, e2, e3, e4, e5, e6]),
        Phase2CeremonyCRS([p0, p1, p2, p3, p4, p5, p6]),
    ))
}

pub fn combine(
    phase1out: &Phase1CeremonyCRS,
    phase2out: &Phase2CeremonyCRS,
    extra: &AllExtraTransitionInformation,
) -> [ProvingKey<Bls12_377>; NUM_CIRCUITS] {
    let [c0, c1, c2, c3, c4, c5, c6] = circuits();
    [
        single::combine(&c0, &phase1out.0[0], &phase2out.0[0], &extra.0[0]),
        single::combine(&c1, &phase1out.0[1], &phase2out.0[1], &extra.0[1]),
        single::combine(&c2, &phase1out.0[2], &phase2out.0[2], &extra.0[2]),
        single::combine(&c3, &phase1out.0[3], &phase2out.0[3], &extra.0[3]),
        single::combine(&c4, &phase1out.0[4], &phase2out.0[4], &extra.0[4]),
        single::combine(&c5, &phase1out.0[5], &phase2out.0[5], &extra.0[5]),
        single::combine(&c6, &phase1out.0[6], &phase2out.0[6], &extra.0[6]),
    ]
}
