use decaf377_fmd::{Clue, Precision};
use penumbra_sdk_keys::Address;
use penumbra_sdk_proto::{core::transaction::v1 as pb, DomainType};

use rand::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct CluePlan {
    pub address: Address,
    pub precision: Precision,
    pub rseed: [u8; 32],
}

impl CluePlan {
    /// Create a new [`CluePlan`] associated with a given (possibly dummy) `Address`.
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        address: Address,
        precision: Precision,
    ) -> CluePlan {
        let mut rseed = [0u8; 32];
        rng.fill_bytes(&mut rseed);
        CluePlan {
            address,
            rseed,
            precision,
        }
    }

    /// Create a [`Clue`] from the [`CluePlan`].
    pub fn clue(&self) -> Clue {
        let clue_key = self.address.clue_key();
        let expanded_clue_key = clue_key.expand_infallible();
        expanded_clue_key
            .create_clue_deterministic(self.precision, self.rseed)
            .expect("can construct clue key")
    }
}

impl DomainType for CluePlan {
    type Proto = pb::CluePlan;
}

impl From<CluePlan> for pb::CluePlan {
    fn from(msg: CluePlan) -> Self {
        Self {
            address: Some(msg.address.into()),
            rseed: msg.rseed.to_vec(),
            precision_bits: msg.precision.bits() as u64,
        }
    }
}

impl TryFrom<pb::CluePlan> for CluePlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::CluePlan) -> Result<Self, Self::Error> {
        Ok(Self {
            address: msg
                .address
                .ok_or_else(|| anyhow::anyhow!("missing address"))?
                .try_into()?,
            rseed: msg.rseed.as_slice().try_into()?,
            precision: msg.precision_bits.try_into()?,
        })
    }
}
