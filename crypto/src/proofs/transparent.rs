//! Transparent proofs for `MVP1` of the Penumbra system.

use anyhow::{anyhow, Context as _, Error, Result};
use ark_ff::{PrimeField, Zero};
use std::convert::{TryFrom, TryInto};

use decaf377::FieldExt;
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_proto::{core::transparent_proofs::v1alpha1 as transparent_proofs, Message, Protobuf};
use penumbra_tct as tct;

use super::transparent_gadgets as gadgets;
use crate::{
    asset, balance,
    dex::{swap::SwapPlaintext, BatchSwapOutputData, TradingPair},
    ka, keys, note,
    transaction::Fee,
    Address, Fq, Fr, Note, Nullifier, Value,
};

/// Transparent proof for spending existing notes.
///
/// This structure keeps track of the auxiliary (private) inputs.
#[derive(Clone, Debug)]
pub struct SpendProof {
    // Inclusion proof for the note commitment.
    pub note_commitment_proof: tct::Proof,
    // The note being spent.
    pub note: Note,
    // The blinding factor used for generating the value commitment.
    pub v_blinding: Fr,
    // The randomizer used for generating the randomized spend auth key.
    pub spend_auth_randomizer: Fr,
    // The spend authorization key.
    pub ak: VerificationKey<SpendAuth>,
    // The nullifier deriving key.
    pub nk: keys::NullifierKey,
}

impl SpendProof {
    /// Called to verify the proof using the provided public inputs.
    ///
    /// The public inputs are:
    /// * the merkle root of the note commitment tree,
    /// * value commitment of the note to be spent,
    /// * nullifier of the note to be spent,
    /// * the randomized verification spend key,
    pub fn verify(
        &self,
        anchor: tct::Root,
        balance_commitment: balance::Commitment,
        nullifier: Nullifier,
        rk: VerificationKey<SpendAuth>,
    ) -> anyhow::Result<()> {
        // Short circuit to true if value released is 0. That means this is a _dummy_ spend.
        if self.note.value().amount == asset::Amount::zero() {
            return Ok(());
        }

        // Note commitment integrity.
        let s_component_transmission_key = self.note.transmission_key_s();
        let note_commitment_test = note::commitment(
            self.note.note_blinding(),
            self.note.value(),
            self.note.diversified_generator(),
            s_component_transmission_key,
            self.note.clue_key(),
        );

        if self.note_commitment_proof.commitment() != note_commitment_test {
            return Err(anyhow!("note commitment mismatch"));
        }

        // Merkle path integrity.
        self.note_commitment_proof
            .verify(anchor)
            .map_err(|_| anyhow!("merkle root mismatch"))?;

        // Value commitment integrity.
        if self.note.value().commit(self.v_blinding) != balance_commitment {
            return Err(anyhow!("value commitment mismatch"));
        }

        // The use of decaf means that we do not need to check that the
        // diversified basepoint is of small order. However we instead
        // check it is not identity.
        if self.note.diversified_generator().is_identity() || self.ak.is_identity() {
            return Err(anyhow!("unexpected identity"));
        }

        gadgets::nullifier_integrity(
            nullifier,
            self.nk,
            self.note_commitment_proof.position(),
            self.note_commitment_proof.commitment(),
        )?;

        // Spend authority.
        let rk_bytes: [u8; 32] = rk.into();
        let rk_test = self.ak.randomize(&self.spend_auth_randomizer);
        let rk_test_bytes: [u8; 32] = rk_test.into();
        if rk_bytes != rk_test_bytes {
            return Err(anyhow!("invalid spend auth randomizer"));
        }

        // Diversified address integrity.
        let fvk = keys::FullViewingKey::from_components(self.ak, self.nk);
        let ivk = fvk.incoming();
        if *self.note.transmission_key()
            != ivk.diversified_public(&self.note.diversified_generator())
        {
            return Err(anyhow!("invalid diversified address"));
        }

        Ok(())
    }
}

/// Transparent proof for new note creation.
///
/// This structure keeps track of the auxiliary (private) inputs.
#[derive(Clone, Debug)]
pub struct OutputProof {
    // The note being created.
    pub note: Note,
    // The blinding factor used for generating the value commitment.
    pub v_blinding: Fr,
    // The ephemeral secret key that corresponds to the public key.
    pub esk: ka::Secret,
}

impl OutputProof {
    /// Called to verify the proof using the provided public inputs.
    ///
    /// The public inputs are:
    /// * value commitment of the new note,
    /// * note commitment of the new note,
    /// * the ephemeral public key used to generate the new note.
    pub fn verify(
        &self,
        balance_commitment: balance::Commitment,
        note_commitment: note::Commitment,
        epk: ka::Public,
    ) -> anyhow::Result<()> {
        // Note commitment integrity.
        let s_component_transmission_key = self.note.transmission_key_s();
        let note_commitment_test = note::commitment(
            self.note.note_blinding(),
            self.note.value(),
            self.note.diversified_generator(),
            s_component_transmission_key,
            self.note.clue_key(),
        );

        if note_commitment != note_commitment_test {
            return Err(anyhow!(
                "note commitment mismatch, public input {:?} does not match witnessed data {:?}",
                note_commitment,
                note_commitment_test,
            ));
        }

        // Value commitment integrity.
        if balance_commitment != -self.note.value().commit(self.v_blinding) {
            return Err(anyhow!("value commitment mismatch"));
        }

        // Ephemeral public key integrity.
        if self
            .esk
            .diversified_public(&self.note.diversified_generator())
            != epk
        {
            return Err(anyhow!("ephemeral public key mismatch"));
        }

        // The use of decaf means that we do not need to check that the
        // diversified basepoint is of small order. However we instead
        // check it is not identity.
        if self.note.address().diversified_generator().is_identity() {
            return Err(anyhow!("unexpected identity"));
        }

        Ok(())
    }
}

// Conversions

impl Protobuf<transparent_proofs::SpendProof> for SpendProof {}

impl From<SpendProof> for transparent_proofs::SpendProof {
    fn from(msg: SpendProof) -> Self {
        let ak_bytes: [u8; 32] = msg.ak.into();
        let nk_bytes: [u8; 32] = msg.nk.0.to_bytes();
        transparent_proofs::SpendProof {
            note_commitment_proof: Some(msg.note_commitment_proof.into()),
            note: Some(msg.note.into()),
            v_blinding: msg.v_blinding.to_bytes().to_vec(),
            spend_auth_randomizer: msg.spend_auth_randomizer.to_bytes().to_vec(),
            ak: ak_bytes.into(),
            nk: nk_bytes.into(),
        }
    }
}

impl TryFrom<transparent_proofs::SpendProof> for SpendProof {
    type Error = Error;

    fn try_from(proto: transparent_proofs::SpendProof) -> anyhow::Result<Self, Self::Error> {
        let v_blinding_bytes: [u8; 32] = proto.v_blinding[..]
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;

        let ak_bytes: [u8; 32] = (proto.ak[..])
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;
        let ak = ak_bytes
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;

        Ok(SpendProof {
            note_commitment_proof: proto
                .note_commitment_proof
                .ok_or_else(|| anyhow!("proto malformed"))?
                .try_into()
                .map_err(|_| anyhow!("proto malformed"))?,
            note: proto
                .note
                .ok_or_else(|| anyhow!("proto malformed"))?
                .try_into()
                .map_err(|_| anyhow!("proto malformed"))?,
            v_blinding: Fr::from_bytes(v_blinding_bytes).map_err(|_| anyhow!("proto malformed"))?,
            spend_auth_randomizer: Fr::from_bytes(
                proto.spend_auth_randomizer[..]
                    .try_into()
                    .map_err(|_| anyhow!("proto malformed"))?,
            )
            .map_err(|_| anyhow!("proto malformed"))?,
            ak,
            nk: keys::NullifierKey(
                Fq::from_bytes(
                    proto.nk[..]
                        .try_into()
                        .map_err(|_| anyhow!("proto malformed"))?,
                )
                .map_err(|_| anyhow!("proto malformed"))?,
            ),
        })
    }
}

impl Protobuf<transparent_proofs::OutputProof> for OutputProof {}

impl From<OutputProof> for transparent_proofs::OutputProof {
    fn from(msg: OutputProof) -> Self {
        transparent_proofs::OutputProof {
            note: Some(msg.note.into()),
            v_blinding: msg.v_blinding.to_bytes().to_vec(),
            esk: msg.esk.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<transparent_proofs::OutputProof> for OutputProof {
    type Error = Error;

    fn try_from(proto: transparent_proofs::OutputProof) -> anyhow::Result<Self, Self::Error> {
        let v_blinding_bytes: [u8; 32] = proto.v_blinding[..]
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;

        let esk_bytes: [u8; 32] = proto.esk[..]
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;
        let esk = ka::Secret::new_from_field(
            Fr::from_bytes(esk_bytes).map_err(|_| anyhow!("proto malformed"))?,
        );

        Ok(OutputProof {
            note: proto
                .note
                .ok_or_else(|| anyhow!("proto malformed"))?
                .try_into()
                .map_err(|_| anyhow!("proto malformed"))?,
            v_blinding: Fr::from_bytes(v_blinding_bytes).map_err(|_| anyhow!("proto malformed"))?,
            esk,
        })
    }
}

impl From<SpendProof> for Vec<u8> {
    fn from(spend_proof: SpendProof) -> Vec<u8> {
        let protobuf_serialized_proof: transparent_proofs::SpendProof = spend_proof.into();
        protobuf_serialized_proof.encode_to_vec()
    }
}

impl TryFrom<&[u8]> for SpendProof {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<SpendProof, Self::Error> {
        let protobuf_serialized_proof = transparent_proofs::SpendProof::decode(bytes)
            .map_err(|_| anyhow!("proto malformed"))?;
        protobuf_serialized_proof
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))
    }
}

impl From<OutputProof> for Vec<u8> {
    fn from(output_proof: OutputProof) -> Vec<u8> {
        let protobuf_serialized_proof: transparent_proofs::OutputProof = output_proof.into();
        protobuf_serialized_proof.encode_to_vec()
    }
}

impl TryFrom<&[u8]> for OutputProof {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<OutputProof, Self::Error> {
        let protobuf_serialized_proof = transparent_proofs::OutputProof::decode(bytes)
            .map_err(|_| anyhow!("proto malformed"))?;
        protobuf_serialized_proof
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))
    }
}

/// Transparent proof for claiming swapped assets.
///
/// SwapClaim consumes an existing Swap NFT so they are most similar to Spend operations,
/// however the note commitment proof needs to be for a specific block due to clearing prices
/// only being valid for particular blocks (i.e. the exchange rates of assets change over time).
///
/// This structure keeps track of the auxiliary (private) inputs.
#[derive(Clone, Debug)]
pub struct SwapClaimProof {
    // Describes the input note with the Swap NFT

    // The asset ID of the swap NFT.
    pub swap_nft_asset_id: asset::Id,
    // The address associated with the swap NFT and outputs.
    pub claim_address: Address,
    // Proves the note commitment was included in the TCT.
    pub note_commitment_proof: tct::Proof,
    // The blinding factor used for generating the note commitment for the Swap NFT.
    pub note_blinding: Fq,
    // The nullifier deriving key for the Swap NFT note.
    pub nk: keys::NullifierKey,

    // Describes opening of Swap NFT asset ID for commitment verification
    pub trading_pair: TradingPair,
    pub delta_1_i: u64,
    pub delta_2_i: u64,

    // Describes output amounts
    pub lambda_1: u64,
    pub lambda_2: u64,

    // Describes first output note (lambda 1)
    pub note_blinding_1: Fq,
    pub esk_1: ka::Secret,

    // Describes second output note (lambda 2)
    pub note_blinding_2: Fq,
    pub esk_2: ka::Secret,
}

impl SwapClaimProof {
    /// Called to verify the proof using the provided public inputs.
    ///
    /// The public inputs are:
    /// * the merkle root of the note commitment tree,
    /// * value commitment of the note to be spent,
    /// * nullifier of the note to be spent,
    /// * the randomized verification spend key,
    /// * the pre-paid fee amount for the swap,
    #[allow(clippy::too_many_arguments)]
    pub fn verify(
        &self,
        anchor: tct::Root,
        nullifier: Nullifier,
        output_data: BatchSwapOutputData,
        epoch_duration: u64,
        note_commitment_1: note::Commitment,
        note_commitment_2: note::Commitment,
        fee: Fee,
        epk_1: ka::Public,
        epk_2: ka::Public,
    ) -> anyhow::Result<()> {
        // Merkle path integrity. Ensure the provided note commitment is in the TCT.
        self.note_commitment_proof
            .verify(anchor)
            .map_err(|_| anyhow!("merkle root mismatch"))?;

        // Check that the provided note commitment is for the proof's Swap NFT.
        let swap_nft_value = Value {
            amount: 1u64.into(),
            asset_id: self.swap_nft_asset_id,
        };
        let transmission_key_s = self.claim_address.transmission_key_s();
        let note_commitment_test = note::commitment(
            self.note_blinding,
            swap_nft_value.clone(),
            *self.claim_address.diversified_generator(),
            *transmission_key_s,
            self.claim_address.clue_key(),
        );

        if self.note_commitment_proof.commitment() != note_commitment_test {
            return Err(anyhow!("note commitment mismatch"));
        }

        // check the swap NFT Asset ID is properly constructed
        let asset_id = self.swap_nft_asset_id;
        let expected_plaintext = SwapPlaintext::from_parts(
            self.trading_pair.clone(),
            self.delta_1_i.into(),
            self.delta_2_i.into(),
            fee,
            // This should ensure that the claim address matches the address
            // used to construct the Swap NFT.
            self.claim_address,
        )
        .map_err(|_| anyhow!("error generating expected swap plaintext"))?;
        let expected_asset_id = expected_plaintext.asset_id();
        if expected_asset_id != asset_id {
            return Err(anyhow!("improper swap NFT asset id"));
        }

        // Validate the note commitment's height matches the output data's height.
        let position = self.note_commitment_proof.position();
        let block = position.block();
        let epoch = position.epoch();
        let note_commitment_block_height: u64 =
            epoch_duration * u64::from(epoch) + u64::from(block);
        if note_commitment_block_height != output_data.height {
            return Err(anyhow::anyhow!(
                "note commitment was not for clearing price height"
            ));
        }

        // Validate that the output data's trading pair matches the note commitment's trading pair.
        if output_data.trading_pair != self.trading_pair {
            return Err(anyhow::anyhow!("trading pair mismatch"));
        }

        // At this point, we've:
        // * verified the note commitment is in the TCT,
        // * verified the note commitment commits to the swap NFT for correct SwapPlaintext,
        // * proved that the prices in the OutputData are for the trading pair at the correct height
        //
        // Now we want to:
        // * spend the swap NFT,
        // * and verify the output notes

        // Swap NFT nullifier integrity. Ensure the nullifier is correctly formed.
        gadgets::nullifier_integrity(
            nullifier,
            self.nk,
            position,
            self.note_commitment_proof.commitment(),
        )?;

        let (lambda_1, lambda_2) = output_data.pro_rata_outputs((self.delta_1_i, self.delta_2_i));

        let value_1 = Value {
            amount: lambda_1.into(),
            asset_id: self.trading_pair.asset_1(),
        };
        let value_2 = Value {
            amount: lambda_2.into(),
            asset_id: self.trading_pair.asset_2(),
        };

        let proof_1 = OutputProof {
            note: Note::from_parts(self.claim_address, value_1, self.note_blinding_1)?,
            // We use a zero blinding factor here because we haven't properly
            // factored out the common code between OutputProof and SwapClaimProof,
            // so we construct a "fake" value commitment internal to this proof.
            v_blinding: Fr::zero(),
            esk: self.esk_1.clone(),
        };
        proof_1
            // This is a dummy value commitment, since we don't have a way of calling the output proof logic otherwise.
            // It has to be -, since an output proof would expect to consume the value
            .verify(-value_1.commit(Fr::zero()), note_commitment_1, epk_1)
            .context("output proof 1 failed")?;

        let proof_2 = OutputProof {
            note: Note::from_parts(self.claim_address, value_2, self.note_blinding_2)?,
            // We use a zero blinding factor here because we haven't properly
            // factored out the common code between OutputProof and SwapClaimProof,
            // so we construct a "fake" value commitment internal to this proof.
            v_blinding: Fr::zero(),
            esk: self.esk_2.clone(),
        };
        proof_2
            // This is a dummy value commitment, since we don't have a way of calling the output proof logic otherwise.
            // It has to be -, since an output proof would expect to consume the value
            .verify(-value_2.commit(Fr::zero()), note_commitment_2, epk_2)
            .context("output proof 2 failed")?;

        Ok(())
    }
}

impl From<SwapClaimProof> for Vec<u8> {
    fn from(swap_proof: SwapClaimProof) -> Vec<u8> {
        let protobuf_serialized_proof: transparent_proofs::SwapClaimProof = swap_proof.into();
        protobuf_serialized_proof.encode_to_vec()
    }
}

impl TryFrom<&[u8]> for SwapClaimProof {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<SwapClaimProof, Self::Error> {
        let protobuf_serialized_proof = transparent_proofs::SwapClaimProof::decode(bytes)
            .map_err(|_| anyhow!("proto malformed"))?;
        protobuf_serialized_proof
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))
    }
}

impl Protobuf<transparent_proofs::SwapClaimProof> for SwapClaimProof {}

impl From<SwapClaimProof> for transparent_proofs::SwapClaimProof {
    fn from(msg: SwapClaimProof) -> Self {
        let nk_bytes: [u8; 32] = msg.nk.0.to_bytes();
        transparent_proofs::SwapClaimProof {
            note_commitment_proof: Some(msg.note_commitment_proof.into()),
            claim_address: Some(msg.claim_address.into()),
            trading_pair: Some(msg.trading_pair.into()),
            delta_1_i: msg.delta_1_i,
            delta_2_i: msg.delta_2_i,
            lambda_1: msg.lambda_1,
            lambda_2: msg.lambda_2,
            note_blinding_1: msg.note_blinding_1.to_bytes().to_vec(),
            note_blinding_2: msg.note_blinding_2.to_bytes().to_vec(),
            esk_1: msg.esk_1.to_bytes().to_vec(),
            esk_2: msg.esk_2.to_bytes().to_vec(),
            swap_nft_asset_id: msg.swap_nft_asset_id.0.to_bytes().to_vec(),
            note_blinding: msg.note_blinding.to_bytes().to_vec(),
            nk: nk_bytes.into(),
        }
    }
}

impl TryFrom<transparent_proofs::SwapClaimProof> for SwapClaimProof {
    type Error = Error;

    fn try_from(proto: transparent_proofs::SwapClaimProof) -> anyhow::Result<Self, Self::Error> {
        let esk_1_bytes: [u8; 32] = proto.esk_1[..]
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;
        let esk_1 = ka::Secret::new_from_field(
            Fr::from_bytes(esk_1_bytes).map_err(|_| anyhow!("proto malformed"))?,
        );
        let esk_2_bytes: [u8; 32] = proto.esk_2[..]
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;
        let esk_2 = ka::Secret::new_from_field(
            Fr::from_bytes(esk_2_bytes).map_err(|_| anyhow!("proto malformed"))?,
        );

        Ok(SwapClaimProof {
            esk_1,
            esk_2,
            note_blinding_1: Fq::from_le_bytes_mod_order(&proto.note_blinding_1),
            note_blinding_2: Fq::from_le_bytes_mod_order(&proto.note_blinding_2),
            lambda_2: proto.lambda_2,
            lambda_1: proto.lambda_1,
            delta_2_i: proto.delta_2_i,
            delta_1_i: proto.delta_1_i,
            trading_pair: proto
                .trading_pair
                .ok_or_else(|| anyhow!("proto malformed"))?
                .try_into()
                .map_err(|_| anyhow!("proto malformed"))?,
            note_commitment_proof: proto
                .note_commitment_proof
                .ok_or_else(|| anyhow!("proto malformed"))?
                .try_into()
                .map_err(|_| anyhow!("proto malformed"))?,
            claim_address: proto
                .claim_address
                .ok_or_else(|| anyhow!("proto malformed"))?
                .try_into()
                .map_err(|_| anyhow!("proto malformed"))?,
            swap_nft_asset_id: asset::Id(
                Fq::from_bytes(
                    proto
                        .swap_nft_asset_id
                        .try_into()
                        .map_err(|_| anyhow!("proto malformed"))?,
                )
                .map_err(|_| anyhow!("proto malformed"))?,
            ),
            note_blinding: Fq::from_bytes(
                proto.note_blinding[..]
                    .try_into()
                    .map_err(|_| anyhow!("proto malformed"))?,
            )
            .map_err(|_| anyhow!("proto malformed"))?,
            nk: keys::NullifierKey(
                Fq::from_bytes(
                    proto.nk[..]
                        .try_into()
                        .map_err(|_| anyhow!("proto malformed"))?,
                )
                .map_err(|_| anyhow!("proto malformed"))?,
            ),
        })
    }
}

/// Transparent proof for swap creation.
///
/// Swaps create an output NFT encoding the swap data so they are most similar to Output operations.
///
/// This structure keeps track of the auxiliary (private) inputs.
#[derive(Clone, Debug)]
pub struct SwapProof {
    // The address associated with the swap.
    pub claim_address: Address,
    // The value of asset 1 in the swap.
    pub value_t1: Value,
    // The value of asset 2 in the swap.
    pub value_t2: Value,
    // The fee amount associated with the swap.
    pub fee_delta: Fee,
    // The blinding factor for the fee.
    pub fee_blinding: Fr,
    // The asset ID of the Swap NFT.
    pub swap_nft_asset_id: asset::Id,
    // The blinding factor used for generating the note commitment for the Swap NFT.
    pub note_blinding: Fq,
    // The ephemeral secret key that corresponds to the public key.
    pub esk: ka::Secret,
    // TODO: no value commitments for delta 1/delta 2 until flow encryption is available
    // // The blinding factor used for generating the value commitment for delta 1.
    // pub delta_1_blinding: Fr,
    // // The blinding factor used for generating the value commitment for delta 2.
    // pub delta_2_blinding: Fr,
}

impl SwapProof {
    /// Called to verify the proof using the provided public inputs.
    ///
    /// The public inputs are:
    /// * value commitment of the asset 1's contribution to the transaction,
    /// * value commitment of the asset 2's contribution to the transaction,
    /// * value commitment of the fee's contribution to the transaction,
    /// * note commitment of the new swap NFT note,
    /// * the ephemeral public key used to generate the new swap NFT note.
    pub fn verify(
        &self,
        _value_1_commitment: balance::Commitment,
        _value_2_commitment: balance::Commitment,
        value_fee_commitment: balance::Commitment,
        note_commitment: note::Commitment,
        epk: ka::Public,
    ) -> anyhow::Result<(), Error> {
        // Note commitment integrity.
        let transmission_key_s = self.claim_address.transmission_key_s();
        // Checks the note commitment of the Swap NFT.
        let note_commitment_test = note::commitment(
            self.note_blinding,
            Value {
                // The swap NFT is always amount 1.
                amount: 1u64.into(),
                asset_id: self.swap_nft_asset_id,
            },
            *self.claim_address.diversified_generator(),
            *transmission_key_s,
            self.claim_address.clue_key(),
        );

        if note_commitment != note_commitment_test {
            return Err(anyhow!("note commitment mismatch"));
        }

        // TODO: no value commitment checks until flow encryption is available
        // // Value commitment integrity.
        // if value_1_commitment != -self.value_t1.commit(self.delta_1_blinding) {
        //     return Err(anyhow!("value commitment mismatch"));
        // }

        // if value_2_commitment != -self.value_t2.commit(self.delta_2_blinding) {
        //     return Err(anyhow!("value commitment mismatch"));
        // }

        if value_fee_commitment != self.fee_delta.commit(self.fee_blinding) {
            return Err(anyhow!("value commitment mismatch"));
        }

        // Ephemeral public key integrity.
        if self
            .esk
            .diversified_public(self.claim_address.diversified_generator())
            != epk
        {
            return Err(anyhow!("ephemeral public key mismatch"));
        }

        // The use of decaf means that we do not need to check that the
        // diversified basepoint is of small order. However we instead
        // check it is not identity.
        if self.claim_address.diversified_generator().is_identity() {
            return Err(anyhow!("unexpected identity"));
        }

        Ok(())
    }
}

impl Protobuf<transparent_proofs::SwapProof> for SwapProof {}

impl From<SwapProof> for transparent_proofs::SwapProof {
    fn from(msg: SwapProof) -> Self {
        transparent_proofs::SwapProof {
            claim_address: Some(msg.claim_address.into()),
            delta_1: msg.value_t1.amount.into(),
            t1: msg.value_t1.asset_id.0.to_bytes().to_vec(),
            delta_2: msg.value_t2.amount.into(),
            t2: msg.value_t2.asset_id.0.to_bytes().to_vec(),
            fee: Some(msg.fee_delta.into()),
            fee_blinding: msg.fee_blinding.to_bytes().to_vec(),
            swap_nft_asset_id: msg.swap_nft_asset_id.0.to_bytes().to_vec(),
            // TODO: no value commitments for delta 1/delta 2 until flow encryption is available
            // delta_1_blinding: msg.delta_1_blinding.to_bytes().to_vec(),
            // delta_2_blinding: msg.delta_2_blinding.to_bytes().to_vec(),
            note_blinding: msg.note_blinding.to_bytes().to_vec(),
            esk: msg.esk.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<transparent_proofs::SwapProof> for SwapProof {
    type Error = Error;

    fn try_from(proto: transparent_proofs::SwapProof) -> anyhow::Result<Self, Self::Error> {
        // let delta_1_blinding_bytes: [u8; 32] = proto.delta_1_blinding[..]
        //     .try_into()
        //     .map_err(|_| anyhow!("proto malformed"))?;
        // let delta_2_blinding_bytes: [u8; 32] = proto.delta_2_blinding[..]
        //     .try_into()
        //     .map_err(|_| anyhow!("proto malformed"))?;

        let fee_blinding_bytes: [u8; 32] = proto.fee_blinding[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("proto malformed"))?;

        let esk_bytes: [u8; 32] = proto.esk[..]
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;
        let esk = ka::Secret::new_from_field(
            Fr::from_bytes(esk_bytes).map_err(|_| anyhow!("proto malformed"))?,
        );

        let _pen_denom = asset::REGISTRY.parse_denom("upenumbra").unwrap();

        Ok(SwapProof {
            claim_address: proto
                .claim_address
                .ok_or_else(|| anyhow!("proto malformed"))?
                .try_into()
                .map_err(|_| anyhow!("proto malformed"))?,
            value_t1: Value {
                amount: proto.delta_1.into(),
                asset_id: asset::Id(
                    Fq::from_bytes(
                        proto
                            .t1
                            .try_into()
                            .map_err(|_| anyhow!("proto malformed"))?,
                    )
                    .map_err(|_| anyhow!("proto malformed"))?,
                ),
            },
            value_t2: Value {
                amount: proto.delta_2.into(),
                asset_id: asset::Id(
                    Fq::from_bytes(
                        proto
                            .t2
                            .try_into()
                            .map_err(|_| anyhow!("proto malformed"))?,
                    )
                    .map_err(|_| anyhow!("proto malformed"))?,
                ),
            },
            fee_delta: proto
                .fee
                .ok_or_else(|| anyhow::anyhow!("proto malformed"))?
                .try_into()
                .map_err(|_| anyhow!("proto malformed"))?,
            fee_blinding: Fr::from_bytes(fee_blinding_bytes)?,
            swap_nft_asset_id: asset::Id(
                Fq::from_bytes(
                    proto
                        .swap_nft_asset_id
                        .try_into()
                        .map_err(|_| anyhow!("proto malformed"))?,
                )
                .map_err(|_| anyhow!("proto malformed"))?,
            ),
            // TODO: no value commitment checks until flow encryption is available
            // delta_1_blinding: Fr::from_bytes(delta_1_blinding_bytes)
            //     .map_err(|_| anyhow!("proto malformed"))?,
            // delta_2_blinding: Fr::from_bytes(delta_2_blinding_bytes)
            //     .map_err(|_| anyhow!("proto malformed"))?,
            note_blinding: Fq::from_bytes(
                proto.note_blinding[..]
                    .try_into()
                    .map_err(|_| anyhow!("proto malformed"))?,
            )
            .map_err(|_| anyhow!("proto malformed"))?,
            esk,
        })
    }
}

impl From<SwapProof> for Vec<u8> {
    fn from(output_proof: SwapProof) -> Vec<u8> {
        let protobuf_serialized_proof: transparent_proofs::SwapProof = output_proof.into();
        protobuf_serialized_proof.encode_to_vec()
    }
}

impl TryFrom<&[u8]> for SwapProof {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<SwapProof, Self::Error> {
        let protobuf_serialized_proof =
            transparent_proofs::SwapProof::decode(bytes).map_err(|_| anyhow!("proto malformed"))?;
        protobuf_serialized_proof
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::UniformRand;
    use rand_core::OsRng;

    use super::*;
    use crate::{
        keys::{SeedPhrase, SpendKey},
        note, Note, Value,
    };

    #[test]
    fn test_output_proof_verification_success() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let value_to_send = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let v_blinding = Fr::rand(&mut rng);
        let note = Note::generate(&mut rng, &dest, value_to_send);
        let esk = ka::Secret::new(&mut rng);
        let epk = esk.diversified_public(&note.diversified_generator());

        let proof = OutputProof {
            note: note.clone(),
            v_blinding,
            esk,
        };

        assert!(proof
            .verify(-value_to_send.commit(v_blinding), note.commit(), epk)
            .is_ok());
    }

    #[test]
    fn test_output_proof_verification_note_commitment_integrity_failure() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let value_to_send = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let v_blinding = Fr::rand(&mut rng);
        let note = Note::generate(&mut rng, &dest, value_to_send);
        let esk = ka::Secret::new(&mut rng);
        let epk = esk.diversified_public(&note.diversified_generator());

        let proof = OutputProof {
            note: note.clone(),
            v_blinding,
            esk,
        };

        let incorrect_note_commitment = note::commitment(
            Fq::rand(&mut rng),
            value_to_send,
            note.diversified_generator(),
            note.transmission_key_s(),
            note.clue_key(),
        );

        assert!(proof
            .verify(
                -value_to_send.commit(v_blinding),
                incorrect_note_commitment,
                epk
            )
            .is_err());
    }

    #[test]
    fn test_output_proof_verification_balance_commitment_integrity_failure() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let value_to_send = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let v_blinding = Fr::rand(&mut rng);
        let note = Note::generate(&mut rng, &dest, value_to_send);
        let esk = ka::Secret::new(&mut rng);
        let correct_epk = esk.diversified_public(&note.diversified_generator());

        let proof = OutputProof {
            note: note.clone(),
            v_blinding,
            esk,
        };
        let incorrect_balance_commitment = value_to_send.commit(Fr::rand(&mut rng));

        assert!(proof
            .verify(incorrect_balance_commitment, note.commit(), correct_epk)
            .is_err());
    }

    #[test]
    fn test_output_proof_verification_ephemeral_public_key_integrity_failure() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let value_to_send = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let v_blinding = Fr::rand(&mut rng);
        let note = Note::generate(&mut rng, &dest, value_to_send);
        let esk = ka::Secret::new(&mut rng);

        let proof = OutputProof {
            note: note.clone(),
            v_blinding,
            esk,
        };
        let incorrect_esk = ka::Secret::new(&mut rng);
        let incorrect_epk = incorrect_esk.diversified_public(&note.diversified_generator());

        assert!(proof
            .verify(
                -value_to_send.commit(v_blinding),
                note.commit(),
                incorrect_epk
            )
            .is_err());
    }

    #[test]
    fn test_spend_proof_verification_success() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u64.into());

        let value_to_send = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let v_blinding = Fr::rand(&mut rng);

        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let spend_auth_randomizer = Fr::rand(&mut rng);
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let mut nct = tct::Tree::new();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = nct.root();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();

        let proof = SpendProof {
            note_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
        };

        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);
        assert!(proof
            .verify(anchor, value_to_send.commit(v_blinding), nf, rk)
            .is_ok());
    }

    #[test]
    fn test_spend_proof_verification_merkle_path_integrity_failure() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u64.into());

        let value_to_send = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let v_blinding = Fr::rand(&mut rng);

        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let spend_auth_randomizer = Fr::rand(&mut rng);
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let mut nct = tct::Tree::new();
        let incorrect_anchor = nct.root();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();

        let proof = SpendProof {
            note_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
        };

        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);
        assert!(proof
            .verify(incorrect_anchor, value_to_send.commit(v_blinding), nf, rk)
            .is_err());
    }

    #[test]
    fn test_spend_proof_verification_balance_commitment_integrity_failure() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u64.into());

        let value_to_send = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let v_blinding = Fr::rand(&mut rng);
        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let spend_auth_randomizer = Fr::rand(&mut rng);
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let mut nct = tct::Tree::new();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = nct.root();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();

        let proof = SpendProof {
            note_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
        };

        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);
        assert!(proof
            .verify(anchor, value_to_send.commit(Fr::rand(&mut rng)), nf, rk)
            .is_err());
    }

    #[test]
    fn test_spend_proof_verification_nullifier_integrity_failure() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u64.into());

        let value_to_send = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let v_blinding = Fr::rand(&mut rng);
        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let spend_auth_randomizer = Fr::rand(&mut rng);
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let mut nct = tct::Tree::new();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = nct.root();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();

        let proof = SpendProof {
            note_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
        };

        let rk: VerificationKey<SpendAuth> = rsk.into();
        let incorrect_nf = nk.derive_nullifier(5.into(), &note_commitment);
        assert!(proof
            .verify(anchor, value_to_send.commit(v_blinding), incorrect_nf, rk)
            .is_err());
    }
}
