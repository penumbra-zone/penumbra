use penumbra_proto::{custody::v1alpha1 as pb, Protobuf};
use penumbra_transaction::plan::TransactionPlan;

/// A pre-authorization packet, containing an Ed25519 signature over a
/// `TransactionPlan`.  This allows a custodian to delegate (partial) signing
/// authority to Ed25519 keys.  Details of how a custodian manages those keys
/// are out-of-scope for the custody protocol itself and are custodian-specific.
#[derive(Debug, Clone)]
pub struct PreAuthorization {
    /// The verification key used to pre-authorize the `TransactionPlan`.
    pub vk: ed25519_consensus::VerificationKey,
    /// An Ed25519 signature over the `TransactionPlan`.
    pub sig: ed25519_consensus::Signature,
}

impl PreAuthorization {
    /// Verifies the provided `TransactionPlan`.
    pub fn verify_plan(&self, plan: &TransactionPlan) -> anyhow::Result<()> {
        let plan_bytes = plan.encode_to_vec();
        self.vk.verify(&self.sig, &plan_bytes).map_err(Into::into)
    }
}

impl Protobuf<pb::PreAuthorization> for PreAuthorization {}

impl TryFrom<pb::PreAuthorization> for PreAuthorization {
    type Error = anyhow::Error;
    fn try_from(value: pb::PreAuthorization) -> Result<Self, Self::Error> {
        Ok(Self {
            vk: value.vk.as_slice().try_into()?,
            sig: value.sig.as_slice().try_into()?,
        })
    }
}

impl From<PreAuthorization> for pb::PreAuthorization {
    fn from(value: PreAuthorization) -> pb::PreAuthorization {
        Self {
            vk: value.vk.to_bytes().into(),
            sig: value.sig.to_bytes().into(),
        }
    }
}
