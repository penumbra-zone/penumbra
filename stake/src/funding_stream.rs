use penumbra_crypto::Address;
use serde::{Deserialize, Serialize};

/// A destination for a portion of a validator's commission of staking rewards.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub struct FundingStream {
    /// The destinatination address for the funding stream..
    pub address: Address,

    /// The portion (in terms of [basis points](https://en.wikipedia.org/wiki/Basis_point)) of the
    /// validator's total staking reward that goes to this funding stream.
    pub rate_bps: u16,
}
