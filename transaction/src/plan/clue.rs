use penumbra_crypto::Address;
use penumbra_proto::{transaction as pb, Protobuf};
use rand::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct CluePlan {
    address: Address,
    rseed: [u8; 32],
}

impl CluePlan {
    /// Create a new [`CluePlan`] associated with a given (possibly dummy) `Address`.
    pub fn new<R: CryptoRng + RngCore>(rng: &mut R, address: Address) -> CluePlan {
        let mut rseed = [0u8; 32];
        rng.fill_bytes(&mut rseed);
        CluePlan { address, rseed }
    }
}

impl Protobuf<pb::CluePlan> for CluePlan {}

impl From<CluePlan> for pb::CluePlan {
    fn from(msg: CluePlan) -> Self {
        Self {
            address: Some(msg.address.into()),
            rseed: msg.rseed.to_vec().into(),
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
            rseed: msg.rseed.as_ref().try_into()?,
        })
    }
}
