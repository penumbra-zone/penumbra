use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::PositionWithdraw;

/// A planned [`PositionWithdraw`](PositionWithdraw).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    try_from = "pb::PositionWithdrawPlan",
    into = "pb::PositionWithdrawPlan"
)]
pub struct PositionWithdrawPlan {}

impl PositionWithdrawPlan {
    /// Create a new [`PositionWithdrawPlan`]
    pub fn new<R: CryptoRng + RngCore>(rng: &mut R) -> PositionWithdrawPlan {
        todo!()
    }

    /// Create a dummy [`PositionWithdrawPlan`].
    pub fn dummy<R: CryptoRng + RngCore>(rng: &mut R) -> PositionWithdrawPlan {
        todo!()
    }

    /// Convenience method to construct the [`PositionWithdraw`] described by this [`PositionWithdrawPlan`].
    pub fn position_withdraw(
        &self,
        // fvk: &FullViewingKey,
        auth_sig: Signature<SpendAuth>,
        auth_path: tct::Proof,
    ) -> PositionWithdraw {
        todo!()
    }

    pub fn balance(&self) -> penumbra_crypto::Balance {
        todo!()
    }
}

impl DomainType for PositionWithdrawPlan {
    type Proto = pb::PositionWithdrawPlan;
}

impl From<PositionWithdrawPlan> for pb::PositionWithdrawPlan {
    fn from(msg: PositionWithdrawPlan) -> Self {
        todo!()
    }
}

impl TryFrom<pb::PositionWithdrawPlan> for PositionWithdrawPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::PositionWithdrawPlan) -> Result<Self, Self::Error> {
        todo!()
    }
}
