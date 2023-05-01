//! Transparent proofs for `MVP1` of the Penumbra system.

use anyhow::{anyhow, ensure, Error, Ok, Result};
use std::convert::{TryFrom, TryInto};

use decaf377::FieldExt;
use penumbra_crypto::{keys, note, Nullifier};
use penumbra_proto::{
    core::transparent_proofs::v1alpha1 as transparent_proofs, DomainType, Message,
};
use penumbra_tct as tct;

use crate::{
    dex::{swap::SwapPlaintext, BatchSwapOutputData},
    keys::{self, NullifierKey},
    note, Amount, Fee, Fq, Nullifier, Value,
};

/// Check the integrity of the nullifier.
pub(crate) fn nullifier_integrity(
    public_nullifier: Nullifier,
    nk: keys::NullifierKey,
    position: tct::Position,
    note_commitment: note::Commitment,
) -> Result<()> {
    if public_nullifier != nk.derive_nullifier(position, &note_commitment) {
        Err(anyhow!("bad nullifier"))
    } else {
        Ok(())
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
    // The swap being claimed
    pub swap_plaintext: SwapPlaintext,
    // Inclusion proof for the swap commitment
    pub swap_commitment_proof: tct::Proof,
    // The nullifier deriving key for the Swap NFT note.
    pub nk: keys::NullifierKey,
    // Describes output amounts
    pub lambda_1_i: Amount,
    pub lambda_2_i: Amount,
}

impl SwapClaimProof {
    /// Called to verify the proof using the provided public inputs.
    ///
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
    ) -> anyhow::Result<()> {
        // Swap commitment integrity
        let swap_commitment = self.swap_plaintext.swap_commitment();
        ensure!(
            swap_commitment == self.swap_commitment_proof.commitment(),
            "swap commitment mismatch"
        );

        // Merkle path integrity. Ensure the provided note commitment is in the TCT.
        self.swap_commitment_proof
            .verify(anchor)
            .map_err(|_| anyhow!("merkle root mismatch"))?;

        // Swap commitment nullifier integrity. Ensure the nullifier is correctly formed.
        nullifier_integrity(
            nullifier,
            self.nk,
            self.swap_commitment_proof.position(),
            self.swap_commitment_proof.commitment(),
        )?;

        // Validate the swap commitment's height matches the output data's height.
        let position = self.swap_commitment_proof.position();
        let block = position.block();
        let epoch = position.epoch();
        let note_commitment_block_height: u64 =
            epoch_duration * u64::from(epoch) + u64::from(block);
        ensure!(
            note_commitment_block_height == output_data.height,
            "note commitment was not for clearing price height"
        );

        // Validate that the output data's trading pair matches the note commitment's trading pair.
        ensure!(
            output_data.trading_pair == self.swap_plaintext.trading_pair,
            "trading pair mismatch"
        );

        // Fee consistency check
        ensure!(fee == self.swap_plaintext.claim_fee, "fee mismatch");

        // Output amounts integrity
        let (lambda_1_i, lambda_2_i) = output_data
            // TODO: Amount conversion ?
            .pro_rata_outputs((
                self.swap_plaintext.delta_1_i.try_into()?,
                self.swap_plaintext.delta_2_i.try_into()?,
            ));
        ensure!(self.lambda_1_i == lambda_1_i, "lambda_1_i mismatch");
        ensure!(self.lambda_2_i == lambda_2_i, "lambda_2_i mismatch");

        // Output note integrity
        let (output_rseed_1, output_rseed_2) = self.swap_plaintext.output_rseeds();
        let output_1_commitment = note::commitment_from_address(
            self.swap_plaintext.claim_address,
            Value {
                amount: self.lambda_1_i.into(),
                asset_id: self.swap_plaintext.trading_pair.asset_1(),
            },
            output_rseed_1.derive_note_blinding(),
        )?;
        let output_2_commitment = note::commitment_from_address(
            self.swap_plaintext.claim_address,
            Value {
                amount: self.lambda_2_i.into(),
                asset_id: self.swap_plaintext.trading_pair.asset_2(),
            },
            output_rseed_2.derive_note_blinding(),
        )?;

        ensure!(
            output_1_commitment == note_commitment_1,
            "output 1 commitment mismatch"
        );
        ensure!(
            output_2_commitment == note_commitment_2,
            "output 2 commitment mismatch"
        );

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

impl DomainType for SwapClaimProof {
    type Proto = transparent_proofs::SwapClaimProof;
}

impl From<SwapClaimProof> for transparent_proofs::SwapClaimProof {
    fn from(msg: SwapClaimProof) -> Self {
        Self {
            swap_commitment_proof: Some(msg.swap_commitment_proof.into()),
            swap_plaintext: Some(msg.swap_plaintext.into()),
            nk: msg.nk.0.to_bytes().to_vec(),
            lambda_1_i: Some(msg.lambda_1_i.into()),
            lambda_2_i: Some(msg.lambda_2_i.into()),
        }
    }
}

impl TryFrom<transparent_proofs::SwapClaimProof> for SwapClaimProof {
    type Error = Error;

    fn try_from(proto: transparent_proofs::SwapClaimProof) -> anyhow::Result<Self, Self::Error> {
        let swap_commitment_proof = proto
            .swap_commitment_proof
            .ok_or_else(|| anyhow!("missing swap commitment proof"))?
            .try_into()?;
        let swap_plaintext = proto
            .swap_plaintext
            .ok_or_else(|| anyhow!("missing swap plaintext"))?
            .try_into()?;
        let nk = NullifierKey(
            Fq::from_bytes(proto.nk.try_into().map_err(|_| anyhow!("invalid nk"))?)
                .map_err(|_| anyhow!("invalid nk"))?,
        );
        let lambda_1_i = proto.lambda_1_i;
        let lambda_2_i = proto.lambda_2_i;

        Ok(Self {
            swap_commitment_proof,
            swap_plaintext,
            nk,
            lambda_1_i: lambda_1_i
                .ok_or_else(|| anyhow!("missing lambda_1_i"))?
                .try_into()?,
            lambda_2_i: lambda_2_i
                .ok_or_else(|| anyhow!("missing lambda_2_i"))?
                .try_into()?,
        })
    }
}
