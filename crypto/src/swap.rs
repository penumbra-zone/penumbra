use crate::ka;
use crate::transaction::Fee;
use anyhow::Result;
use penumbra_proto::{dex as pb, Protobuf};

use crate::asset::Id as AssetId;
use crate::keys::OutgoingViewingKey;

// TODO: not sure what byte size is necessary here yet
pub const SWAP_CIPHERTEXT_BYTES: usize = 128;

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

impl SwapPlaintext {
    // TODO: needs to be constant-length
    pub fn encrypt(&self, ovk: &OutgoingViewingKey) -> SwapCiphertext {
        let shared_secret = ovk
            .key_agreement_with(&self.transmission_key())
            .expect("key agreement succeeded");

        let key = derive_symmetric_key(&shared_secret, &epk);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
        let nonce = Nonce::from_slice(&*NOTE_ENCRYPTION_NONCE);

        let note_plaintext: Vec<u8> = self.into();
        let encryption_result = cipher
            .encrypt(nonce, note_plaintext.as_ref())
            .expect("note encryption succeeded");

        let ciphertext: [u8; NOTE_CIPHERTEXT_BYTES] = encryption_result
            .try_into()
            .expect("note encryption result fits in ciphertext len");

        ciphertext
    }
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
            fee: Fee(plaintext
                .fee
                .ok_or_else(|| anyhow::anyhow!("missing SwapPlaintext fee"))?
                .amount),
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
            fee: Some(penumbra_proto::transaction::Fee {
                amount: plaintext.fee.0,
            }),
            b_d: plaintext.b_d.compress().0.to_vec(),
            pk_d: plaintext.pk_d.0.to_vec(),
            trading_pair: Some(plaintext.trading_pair.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapCiphertext(pub [u8; SWAP_CIPHERTEXT_BYTES]);

impl SwapCiphertext {
    // TODO: needs to be constant-length
    pub fn decrypt(&self, _ovk: &OutgoingViewingKey) -> Result<SwapPlaintext> {
        // TODO: implement
        Err(anyhow::anyhow!("not implemented"))
    }
}

impl TryFrom<[u8; SWAP_CIPHERTEXT_BYTES]> for SwapCiphertext {
    type Error = anyhow::Error;

    fn try_from(bytes: [u8; SWAP_CIPHERTEXT_BYTES]) -> Result<SwapCiphertext, Self::Error> {
        Ok(SwapCiphertext(bytes))
    }
}

impl TryFrom<&[u8]> for SwapCiphertext {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<SwapCiphertext, Self::Error> {
        Ok(SwapCiphertext(slice[..].try_into()?))
    }
}

// TODO: ideally this would live in `component/dex/` or maybe a new subcrate including only
// dex-related logic, but the former causes cyclic import issues
#[derive(Debug, Clone)]
pub struct TradingPair {
    pub asset_1: AssetId,
    pub asset_2: AssetId,
}

impl Protobuf<pb::TradingPair> for TradingPair {}

impl TryFrom<pb::TradingPair> for TradingPair {
    type Error = anyhow::Error;
    fn try_from(tp: pb::TradingPair) -> anyhow::Result<Self> {
        Ok(Self {
            asset_1: tp
                .asset_1
                .ok_or_else(|| anyhow::anyhow!("missing trading pair asset1"))?
                .try_into()?,
            asset_2: tp
                .asset_2
                .ok_or_else(|| anyhow::anyhow!("missing trading pair asset2"))?
                .try_into()?,
        })
    }
}

impl From<TradingPair> for pb::TradingPair {
    fn from(tp: TradingPair) -> Self {
        Self {
            asset_1: Some(tp.asset_1.into()),
            asset_2: Some(tp.asset_2.into()),
        }
    }
}
