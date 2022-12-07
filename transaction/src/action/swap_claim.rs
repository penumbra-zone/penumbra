use crate::view::action_view::SwapClaimView;
use crate::{ActionView, IsAction, TransactionPerspective};
use ark_ff::Zero;
use penumbra_crypto::dex::BatchSwapOutputData;
use penumbra_crypto::transaction::Fee;
use penumbra_crypto::{proofs::transparent::SwapClaimProof, EncryptedNote, Fr};
use penumbra_crypto::{Balance, Note, Nullifier};
use penumbra_proto::{core::dex::v1alpha1 as pb, Protobuf};

#[derive(Debug, Clone)]
pub struct SwapClaim {
    pub proof: SwapClaimProof,
    pub body: Body,
}

impl IsAction for SwapClaim {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, txp: &TransactionPerspective) -> ActionView {
        // For each note payload (output_1, output_2)
        let note_commitment_1 = self.body.output_1.note_commitment;
        let note_commitment_2 = self.body.output_2.note_commitment;
        // Get payload key for note commitment of note payload
        let payload_key_1 = txp.payload_keys.get(&note_commitment_1);
        let payload_key_2 = txp.payload_keys.get(&note_commitment_2);

        let swap_claim_view = if let (Some(payload_key_1), Some(payload_key_2)) =
            (payload_key_1, payload_key_2)
        {
            // * Decrypt notes
            let decrypted_note_1 =
                Note::decrypt_with_payload_key(&self.body.output_1.encrypted_note, payload_key_1);
            let decrypted_note_2 =
                Note::decrypt_with_payload_key(&self.body.output_2.encrypted_note, payload_key_2);

            if let (Ok(decrypted_note_1), Ok(decrypted_note_2)) =
                (decrypted_note_1, decrypted_note_2)
            {
                SwapClaimView::Visible {
                    swap_claim: self.to_owned(),
                    output_1: decrypted_note_1,
                    output_2: decrypted_note_2,
                }
            } else {
                SwapClaimView::Opaque {
                    swap_claim: self.to_owned(),
                }
            }
        } else {
            SwapClaimView::Opaque {
                swap_claim: self.to_owned(),
            }
        };

        ActionView::SwapClaim(swap_claim_view)
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
    pub output_1: EncryptedNote,
    pub output_2: EncryptedNote,
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
