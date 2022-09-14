use crate::transaction_view::action_view::SwapClaimView;
use crate::{ActionView, IsAction, TransactionPerspective};
use ark_ff::Zero;
use penumbra_crypto::dex::BatchSwapOutputData;
use penumbra_crypto::transaction::Fee;
use penumbra_crypto::{proofs::transparent::SwapClaimProof, Fr, NotePayload};
use penumbra_crypto::{Balance, Note, Nullifier};
use penumbra_proto::{core::dex::v1alpha1 as pb, Protobuf};
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct SwapClaim {
    pub proof: SwapClaimProof,
    pub body: Body,
}

impl IsAction for SwapClaim {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn decrypt_with_perspective(
        &self,
        txp: &TransactionPerspective,
    ) -> anyhow::Result<Option<crate::ActionView>> {
        // For each note payload (output_1, output_2)
        let note_commitment_1 = self.body.output_1.note_commitment;
        let note_commitment_2 = self.body.output_2.note_commitment;
        // Get payload key for note commitment of note payload
        let payload_key_1 = txp
            .payload_keys
            .get(&note_commitment_1)
            .ok_or_else(|| anyhow::anyhow!("corresponding payload key not found"))?;
        let payload_key_2 = txp
            .payload_keys
            .get(&note_commitment_2)
            .ok_or_else(|| anyhow::anyhow!("corresponding payload key not found"))?;
        // * Decrypt notes
        let decrypted_note_1 =
            Note::decrypt_with_payload_key(&self.body.output_1.encrypted_note, payload_key_1)?;
        let decrypted_note_2 =
            Note::decrypt_with_payload_key(&self.body.output_2.encrypted_note, payload_key_2)?;

        Ok(Some(ActionView::SwapClaim(SwapClaimView {
            decrypted_note_1,
            decrypted_note_2,
        })))
    }
}

impl SwapClaim {
    /// Compute a commitment to the value contributed to a transaction by this swap claim.
    /// Will add (f,fee_token) representing the pre-paid fee
    pub fn balance(&self) -> Balance {
        self.body.fee.value().into()
    }
}

impl Protobuf<pb::SwapClaim> for SwapClaim {}

impl From<SwapClaim> for pb::SwapClaim {
    fn from(sc: SwapClaim) -> Self {
        pb::SwapClaim {
            proof: sc.proof.into(),
            body: Some(sc.body.into()),
        }
    }
}

impl TryFrom<pb::SwapClaim> for SwapClaim {
    type Error = anyhow::Error;
    fn try_from(sc: pb::SwapClaim) -> Result<Self, Self::Error> {
        Ok(Self {
            proof: sc.proof[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("SwapClaim proof malformed"))?,
            body: sc
                .body
                .ok_or_else(|| anyhow::anyhow!("missing nullifier"))?
                .try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Body {
    pub nullifier: Nullifier,
    pub fee: Fee,
    pub output_1: NotePayload,
    pub output_2: NotePayload,
    pub output_data: BatchSwapOutputData,
    pub epoch_duration: u64,
}

impl Protobuf<pb::SwapClaimBody> for Body {}

impl From<Body> for pb::SwapClaimBody {
    fn from(s: Body) -> Self {
        pb::SwapClaimBody {
            nullifier: Some(s.nullifier.into()),
            fee: Some(s.fee.into()),
            output_1: Some(s.output_1.into()),
            output_2: Some(s.output_2.into()),
            output_data: Some(s.output_data.into()),
            epoch_duration: s.epoch_duration,
        }
    }
}

impl TryFrom<pb::SwapClaimBody> for Body {
    type Error = anyhow::Error;
    fn try_from(sc: pb::SwapClaimBody) -> Result<Self, Self::Error> {
        Ok(Self {
            nullifier: sc
                .nullifier
                .ok_or_else(|| anyhow::anyhow!("missing nullifier"))?
                .try_into()?,
            fee: sc
                .fee
                .ok_or_else(|| anyhow::anyhow!("missing fee"))?
                .try_into()?,
            output_1: sc
                .output_1
                .ok_or_else(|| anyhow::anyhow!("missing output_1"))?
                .try_into()?,
            output_2: sc
                .output_2
                .ok_or_else(|| anyhow::anyhow!("missing output_2"))?
                .try_into()?,
            output_data: sc
                .output_data
                .ok_or_else(|| anyhow::anyhow!("missing anchor"))?
                .try_into()?,
            epoch_duration: sc.epoch_duration,
        })
    }
}

// Represents a swap claimed in a particular transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::ClaimedSwap", into = "pb::ClaimedSwap")]
pub struct ClaimedSwap(pub Body, pub [u8; 32]);

impl Protobuf<pb::ClaimedSwap> for ClaimedSwap {}

impl TryFrom<pb::ClaimedSwap> for ClaimedSwap {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ClaimedSwap) -> Result<Self, Self::Error> {
        let txid_bytes: [u8; 32] = msg.txid[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("proto malformed"))?;

        Ok(ClaimedSwap(
            msg.claim
                .ok_or_else(|| anyhow::anyhow!("proto malformed"))?
                .try_into()
                .map_err(|_| anyhow::anyhow!("proto malformed"))?,
            txid_bytes,
        ))
    }
}

impl From<ClaimedSwap> for pb::ClaimedSwap {
    fn from(vk: ClaimedSwap) -> Self {
        pb::ClaimedSwap {
            claim: Some(vk.0.into()),
            txid: vk.1.to_vec(),
        }
    }
}

/// A list of swap claim bodies.
///
/// This is a newtype wrapper for a Vec that allows us to define a proto type.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "pb::ClaimedSwapList", into = "pb::ClaimedSwapList")]
pub struct List(pub Vec<ClaimedSwap>);

impl Protobuf<pb::ClaimedSwapList> for List {}

impl TryFrom<pb::ClaimedSwapList> for List {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ClaimedSwapList) -> Result<Self, Self::Error> {
        Ok(List(
            msg.claims
                .iter()
                .map(|claim| claim.clone().try_into())
                .collect::<anyhow::Result<Vec<_>>>()?,
        ))
    }
}

impl From<List> for pb::ClaimedSwapList {
    fn from(vk: List) -> Self {
        pb::ClaimedSwapList {
            claims: vk.0.iter().map(|v| v.clone().into()).collect(),
        }
    }
}
