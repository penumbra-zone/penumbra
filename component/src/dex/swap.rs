use decaf377::Element;
use penumbra_crypto::{asset, ka};
use penumbra_proto::{dex as pb, Protobuf};
use penumbra_transaction::Fee;

use super::TradingPair;

#[derive(Clone)]
pub struct SwapPlaintext {
    // Trading pair for the swap
    pub trading_pair: TradingPair,
    // Amount of asset 1
    pub t1: u64,
    // Amount of asset 2
    pub t2: u64,
    // Fee
    pub fee: Fee,
    // Diversified basepoint
    pub b_d: decaf377::Element,
    // Diversified public key
    pub pk_d: ka::Public,
}

impl Protobuf<pb::SwapPlaintext> for SwapPlaintext {}

impl TryFrom<pb::SwapPlaintext> for SwapPlaintext {
    type Error = anyhow::Error;
    fn try_from(plaintext: pb::SwapPlaintext) -> anyhow::Result<Self> {
        let b_d_bytes: [u8; 32] = plaintext
            .b_d
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid diversified basepoint in SwapPlaintext"))?;
        let b_d_encoding = decaf377::Encoding(b_d_bytes);

        Ok(Self {
            t1: plaintext.t1,
            t2: plaintext.t2,
            fee: plaintext
                .fee
                .ok_or_else(|| anyhow::anyhow!("missing SwapPlaintext fee"))?
                .try_into()?,
            b_d: b_d_encoding.decompress().map_err(|_| {
                anyhow::anyhow!("error decompressing diversified basepoint in SwapPlaintext")
            })?,
            pk_d: ka::Public(
                plaintext.pk_d.try_into().map_err(|_| {
                    anyhow::anyhow!("invalid diversified publickey in SwapPlaintext")
                })?,
            ),
            trading_pair: plaintext
                .trading_pair
                .ok_or_else(|| anyhow::anyhow!("missing trading pair in SwapPlaintext"))?
                .try_into()?,
        })
    }
}

impl From<SwapPlaintext> for pb::SwapPlaintext {
    fn from(plaintext: SwapPlaintext) -> Self {
        Self {
            t1: plaintext.t1,
            t2: plaintext.t2,
            fee: Some(plaintext.fee.into()),
            b_d: plaintext.b_d.compress().0.to_vec(),
            pk_d: plaintext.pk_d.0.to_vec(),
            trading_pair: Some(plaintext.trading_pair.into()),
        }
    }
}
