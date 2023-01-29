use penumbra_proto::{custody::v1alpha1 as pb, Protobuf};
use penumbra_transaction::plan::TransactionPlan;
use serde::{Deserialize, Serialize};

/// A pre-authorization packet.  This allows a custodian to delegate (partial)
/// signing authority to other authorization mechanisms.  Details of how a
/// custodian manages those keys are out-of-scope for the custody protocol and
/// are custodian-specific.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::PreAuthorization", into = "pb::PreAuthorization")]
pub enum PreAuthorization {
    Ed25519(Ed25519),
}

/// An Ed25519-based preauthorization, containing an Ed25519 signature over the
/// `TransactionPlan`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::pre_authorization::Ed25519",
    into = "pb::pre_authorization::Ed25519"
)]
pub struct Ed25519 {
    /// The verification key used to pre-authorize the `TransactionPlan`.
    pub vk: ed25519_consensus::VerificationKey,
    /// An Ed25519 signature over the `TransactionPlan`.
    pub sig: ed25519_consensus::Signature,
}

impl Ed25519 {
    /// Verifies the provided `TransactionPlan`.
    pub fn verify_plan(&self, plan: &TransactionPlan) -> anyhow::Result<()> {
        let plan_bytes = plan.encode_to_vec();
        self.vk.verify(&self.sig, &plan_bytes).map_err(Into::into)
    }
}

impl Protobuf for PreAuthorization {
    type Proto = pb::PreAuthorization;
}

impl TryFrom<pb::PreAuthorization> for PreAuthorization {
    type Error = anyhow::Error;
    fn try_from(value: pb::PreAuthorization) -> Result<Self, Self::Error> {
        Ok(match value.pre_authorization {
            Some(pb::pre_authorization::PreAuthorization::Ed25519(ed)) => {
                Self::Ed25519(ed.try_into()?)
            }
            None => {
                return Err(anyhow::anyhow!("missing pre-authorization"));
            }
        })
    }
}

impl From<PreAuthorization> for pb::PreAuthorization {
    fn from(value: PreAuthorization) -> pb::PreAuthorization {
        Self {
            pre_authorization: Some(match value {
                PreAuthorization::Ed25519(ed) => {
                    pb::pre_authorization::PreAuthorization::Ed25519(ed.into())
                }
            }),
        }
    }
}

impl Protobuf for Ed25519 {
    type Proto = pb::pre_authorization::Ed25519;
}

impl TryFrom<pb::pre_authorization::Ed25519> for Ed25519 {
    type Error = anyhow::Error;
    fn try_from(value: pb::pre_authorization::Ed25519) -> Result<Self, Self::Error> {
        Ok(Self {
            vk: value.vk.as_slice().try_into()?,
            sig: value.sig.as_slice().try_into()?,
        })
    }
}

impl From<Ed25519> for pb::pre_authorization::Ed25519 {
    fn from(value: Ed25519) -> pb::pre_authorization::Ed25519 {
        Self {
            vk: value.vk.to_bytes().into(),
            sig: value.sig.to_bytes().into(),
        }
    }
}
