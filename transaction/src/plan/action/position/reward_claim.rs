use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_tct as tct;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::PositionRewardClaim;

/// A planned [`PositionRewardClaim`](PositionRewardClaim).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    try_from = "pb::PositionRewardClaimPlan",
    into = "pb::PositionRewardClaimPlan"
)]
pub struct PositionRewardClaimPlan {}

impl PositionRewardClaimPlan {
    /// Create a new [`PositionRewardClaimPlan`]
    pub fn new<R: CryptoRng + RngCore>(_rng: &mut R) -> PositionRewardClaimPlan {
        todo!()
    }

    /// Create a dummy [`PositionRewardClaimPlan`].
    pub fn dummy<R: CryptoRng + RngCore>(_rng: &mut R) -> PositionRewardClaimPlan {
        todo!()
    }

    /// Convenience method to construct the [`PositionRewardClaim`] described by this [`PositionRewardClaimPlan`].
    pub fn position_reward_claim(
        &self,
        // fvk: &FullViewingKey,
        _auth_sig: Signature<SpendAuth>,
        _auth_path: tct::Proof,
    ) -> PositionRewardClaim {
        todo!()
    }

    pub fn balance(&self) -> penumbra_crypto::Balance {
        todo!()
    }
}

impl TypeUrl for PositionRewardClaimPlan {
    const TYPE_URL: &'static str = "/penumbra.core.dex.v1alpha1.PositionRewardClaimPlan";
}

impl DomainType for PositionRewardClaimPlan {
    type Proto = pb::PositionRewardClaimPlan;
}

impl From<PositionRewardClaimPlan> for pb::PositionRewardClaimPlan {
    fn from(_msg: PositionRewardClaimPlan) -> Self {
        todo!()
    }
}

impl TryFrom<pb::PositionRewardClaimPlan> for PositionRewardClaimPlan {
    type Error = anyhow::Error;
    fn try_from(_msg: pb::PositionRewardClaimPlan) -> Result<Self, Self::Error> {
        todo!()
    }
}
