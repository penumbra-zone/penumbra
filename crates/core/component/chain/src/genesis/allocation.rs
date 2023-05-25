use penumbra_crypto::{
    asset::{self, Denom},
    Address, Amount, Note, Rseed, Value,
};
use penumbra_proto::{core::chain::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

/// A (transparent) genesis allocation.
#[derive(Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::genesis_app_state::Allocation",
    into = "pb::genesis_app_state::Allocation"
)]
pub struct Allocation {
    pub amount: Amount,
    pub denom: String,
    pub address: Address,
}

impl From<Allocation> for pb::genesis_app_state::Allocation {
    fn from(a: Allocation) -> Self {
        pb::genesis_app_state::Allocation {
            amount: Some(a.amount.into()),
            denom: a.denom,
            address: Some(a.address.into()),
        }
    }
}

impl TryFrom<pb::genesis_app_state::Allocation> for Allocation {
    type Error = anyhow::Error;

    fn try_from(msg: pb::genesis_app_state::Allocation) -> Result<Self, Self::Error> {
        Ok(Allocation {
            amount: msg
                .amount
                .ok_or_else(|| anyhow::anyhow!("missing amount field in proto"))?
                .try_into()?,
            denom: msg.denom,
            address: msg
                .address
                .ok_or_else(|| anyhow::anyhow!("missing address field in proto"))?
                .try_into()?,
        })
    }
}

// Implement Debug manually so we can use the Display impl for the address,
// rather than dumping all the internal address components.
impl std::fmt::Debug for Allocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Allocation")
            .field("amount", &self.amount)
            .field("denom", &self.denom)
            .field("address", &self.address.to_string())
            .finish()
    }
}

impl Allocation {
    /// Obtain a note corresponding to this allocation.
    ///
    /// Note: to ensure determinism, this uses a zero rseed when
    /// creating the note.
    pub fn note(&self) -> Result<Note, anyhow::Error> {
        Note::from_parts(
            self.address,
            Value {
                amount: self.amount,
                asset_id: asset::DenomMetadata::default_for(&Denom {
                    denom: self.denom.clone(),
                })
                .unwrap()
                .id(),
            },
            Rseed([0u8; 32]),
        )
        .map_err(Into::into)
    }
}

impl TypeUrl for Allocation {
    // TODO: verify!
    const TYPE_URL: &'static str = "/penumbra.core.chain.v1alpha1.genesis_app_state.Allocation";
}

impl DomainType for Allocation {
    type Proto = pb::genesis_app_state::Allocation;
}
