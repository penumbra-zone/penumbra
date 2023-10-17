use anyhow::Result;
use async_trait::async_trait;
use penumbra_keys::Address;
use penumbra_proof_setup::all::{
    Phase1CeremonyCRS, Phase1CeremonyContribution, Phase1RawCeremonyContribution,
    Phase2CeremonyCRS, Phase2CeremonyContribution, Phase2RawCeremonyContribution,
};
use penumbra_proto::tools::summoning::v1alpha1::{
    participate_request::Contribution as PBContribution, CeremonyCrs,
};
use rand_core::CryptoRngCore;

use crate::storage::Storage;

/// A simple marker for which phase we're in, which some code can depend on at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseMarker {
    P1,
    P2,
}

/// A utility trait to exist solely for plumbing around code that's varies with phases.
///
/// This contains some types and constants, along with various stub methods that are
/// simple one-liners, or should be.
#[async_trait]
pub trait Phase {
    /// The type for the elements.
    type CRS: Clone + Send + Sync;

    /// The type for unvalidated contributions.
    type RawContribution: Send + Sync;

    /// The type for validated contributions.
    type Contribution: Send + Sync;

    /// The constant value for the marker we use, for runtime dispatch.
    const MARKER: PhaseMarker;

    /// The amount of time we should wait for a contribution.
    ///
    /// This varies since one phase is more expensive than the other.
    const CONTRIBUTION_TIME_SECS: u64;

    /// Serialize the CRS value, in a potentially failing way.
    fn serialize_crs(data: Self::CRS) -> Result<CeremonyCrs>;

    /// Deserialize a contribution, without validation.
    fn deserialize_contribution(data: PBContribution) -> Result<Self::RawContribution>;

    /// Validate a contribution, using some randomness, and the root for the phase.
    ///
    /// Note: this can be expensive.
    fn validate(
        rng: &mut impl CryptoRngCore,
        root: &Self::CRS,
        contribution: Self::RawContribution,
    ) -> Option<Self::Contribution>;

    /// Check if a contribution is linked to some parent elements.
    fn is_linked_to(contribution: &Self::Contribution, parent: &Self::CRS) -> bool;

    /// Fetch the root for this phase from storage.
    async fn fetch_root(storage: &Storage) -> Result<Self::CRS>;

    /// Fetch the latest elements for this phase from storage.
    async fn current_crs(storage: &Storage) -> Result<Option<Self::CRS>>;

    /// Commit a contribution to the right phase table in storage.
    async fn commit_contribution(
        storage: &Storage,
        contributor: Address,
        contribution: Self::Contribution,
    ) -> Result<()>;
}

pub struct Phase1;

#[async_trait]
impl Phase for Phase1 {
    type CRS = Phase1CeremonyCRS;
    type RawContribution = Phase1RawCeremonyContribution;
    type Contribution = Phase1CeremonyContribution;
    const MARKER: PhaseMarker = PhaseMarker::P1;
    const CONTRIBUTION_TIME_SECS: u64 = 20 * 60;

    fn serialize_crs(data: Self::CRS) -> Result<CeremonyCrs> {
        data.try_into()
    }

    fn deserialize_contribution(data: PBContribution) -> Result<Self::RawContribution> {
        data.try_into()
    }

    fn validate(
        rng: &mut impl CryptoRngCore,
        _root: &Self::CRS,
        contribution: Self::RawContribution,
    ) -> Option<Self::Contribution> {
        contribution.validate(rng)
    }

    fn is_linked_to(contribution: &Self::Contribution, parent: &Self::CRS) -> bool {
        contribution.is_linked_to(parent)
    }

    async fn fetch_root(storage: &Storage) -> Result<Self::CRS> {
        Ok(storage.phase1_root().await?)
    }

    async fn current_crs(storage: &Storage) -> Result<Option<Self::CRS>> {
        Ok(storage.phase1_current_crs().await?)
    }

    async fn commit_contribution(
        storage: &Storage,
        contributor: Address,
        contribution: Self::Contribution,
    ) -> Result<()> {
        Ok(storage
            .phase1_commit_contribution(contributor, contribution)
            .await?)
    }
}

pub struct Phase2;

#[async_trait]
impl Phase for Phase2 {
    type CRS = Phase2CeremonyCRS;
    type RawContribution = Phase2RawCeremonyContribution;
    type Contribution = Phase2CeremonyContribution;
    const MARKER: PhaseMarker = PhaseMarker::P2;
    const CONTRIBUTION_TIME_SECS: u64 = 10 * 60;

    fn serialize_crs(data: Self::CRS) -> Result<CeremonyCrs> {
        data.try_into()
    }

    fn deserialize_contribution(data: PBContribution) -> Result<Self::RawContribution> {
        data.try_into()
    }

    fn validate(
        rng: &mut impl CryptoRngCore,
        root: &Self::CRS,
        contribution: Self::RawContribution,
    ) -> Option<Self::Contribution> {
        contribution.validate(rng, root)
    }

    fn is_linked_to(contribution: &Self::Contribution, parent: &Self::CRS) -> bool {
        contribution.is_linked_to(parent)
    }

    async fn fetch_root(storage: &Storage) -> Result<Self::CRS> {
        Ok(storage.phase2_root().await?)
    }

    async fn current_crs(storage: &Storage) -> Result<Option<Self::CRS>> {
        Ok(storage.phase2_current_crs().await?)
    }

    async fn commit_contribution(
        storage: &Storage,
        contributor: Address,
        contribution: Self::Contribution,
    ) -> Result<()> {
        Ok(storage
            .phase2_commit_contribution(contributor, contribution)
            .await?)
    }
}
