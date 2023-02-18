//! Transparent proofs for `MVP1` of the Penumbra system.

use anyhow::{anyhow, ensure, Error, Ok, Result};
use ark_ff::Zero;
use std::convert::{TryFrom, TryInto};

use decaf377::FieldExt;
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_proto::{
    core::transparent_proofs::v1alpha1 as transparent_proofs, DomainType, Message,
};
use penumbra_tct as tct;

use super::transparent_gadgets as gadgets;
use crate::{
    asset, balance,
    dex::{swap::SwapPlaintext, BatchSwapOutputData},
    keys::{self, NullifierKey},
    note,
    stake::Penalty,
    transaction::Fee,
    Amount, Balance, Fq, Fr, Note, Nullifier, Value,
};

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
    pub lambda_1_i: u64,
    pub lambda_2_i: u64,
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
        gadgets::nullifier_integrity(
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
            lambda_1_i: msg.lambda_1_i,
            lambda_2_i: msg.lambda_2_i,
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
            lambda_1_i,
            lambda_2_i,
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
    pub swap_plaintext: SwapPlaintext,
    // The blinding factor for the fee.
    pub fee_blinding: Fr,
}

impl SwapProof {
    /// Called to verify the proof using the provided public inputs.
    pub fn verify(
        &self,
        fee_commitment: balance::Commitment,
        swap_commitment: tct::Commitment,
        balance_commitment: balance::Commitment,
    ) -> anyhow::Result<(), Error> {
        // Swap commitment integrity check
        ensure!(
            swap_commitment == self.swap_plaintext.swap_commitment(),
            "swap commitment mismatch"
        );

        // Fee commitment integrity check
        ensure!(
            fee_commitment == self.swap_plaintext.claim_fee.commit(self.fee_blinding),
            "fee commitment mismatch"
        );

        // Now reconstruct the swap action's balance commitment
        let transparent_balance = Balance::default()
            - Value {
                amount: self.swap_plaintext.delta_1_i,
                asset_id: self.swap_plaintext.trading_pair.asset_1(),
            }
            - Value {
                amount: self.swap_plaintext.delta_2_i,
                asset_id: self.swap_plaintext.trading_pair.asset_2(),
            };
        let transparent_balance_commitment = transparent_balance.commit(Fr::zero());

        // XXX we want to avoid having to twiddle signs for synthetic blinding factor in binding sig
        ensure!(
            balance_commitment == transparent_balance_commitment + fee_commitment,
            "balance commitment mismatch"
        );

        Ok(())
    }
}

impl DomainType for SwapProof {
    type Proto = transparent_proofs::SwapProof;
}

impl From<SwapProof> for transparent_proofs::SwapProof {
    fn from(msg: SwapProof) -> Self {
        transparent_proofs::SwapProof {
            swap_plaintext: Some(msg.swap_plaintext.into()),
            fee_blinding: msg.fee_blinding.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<transparent_proofs::SwapProof> for SwapProof {
    type Error = Error;

    fn try_from(proto: transparent_proofs::SwapProof) -> anyhow::Result<Self, Self::Error> {
        let swap_plaintext = proto
            .swap_plaintext
            .ok_or_else(|| anyhow!("proto malformed"))?
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;
        let fee_blinding = Fr::from_bytes(
            proto.fee_blinding[..]
                .try_into()
                .map_err(|_| anyhow!("proto malformed"))?,
        )
        .map_err(|_| anyhow!("proto malformed"))?;

        Ok(SwapProof {
            swap_plaintext,
            fee_blinding,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndelegateClaimProof {
    unbonding_amount: Amount,
    balance_blinding: Fr,
}

impl UndelegateClaimProof {
    pub fn new(unbonding_amount: Amount, balance_blinding: Fr) -> Self {
        Self {
            unbonding_amount,
            balance_blinding,
        }
    }

    pub fn verify(
        &self,
        balance_commitment: balance::Commitment,
        unbonding_id: asset::Id,
        penalty: Penalty,
    ) -> anyhow::Result<()> {
        let expected_balance = penalty.balance_for_claim(unbonding_id, self.unbonding_amount);
        let expected_commitment = expected_balance.commit(self.balance_blinding);
        ensure!(
            expected_commitment == balance_commitment,
            "balance commitment mismatch"
        );
        Ok(())
    }
}

impl DomainType for UndelegateClaimProof {
    type Proto = transparent_proofs::UndelegateClaimProof;
}

impl From<UndelegateClaimProof> for transparent_proofs::UndelegateClaimProof {
    fn from(claim_proof: UndelegateClaimProof) -> Self {
        transparent_proofs::UndelegateClaimProof {
            unbonding_amount: Some(claim_proof.unbonding_amount.into()),
            balance_blinding: claim_proof.balance_blinding.to_bytes().into(),
        }
    }
}

impl TryFrom<transparent_proofs::UndelegateClaimProof> for UndelegateClaimProof {
    type Error = Error;

    fn try_from(proto: transparent_proofs::UndelegateClaimProof) -> Result<Self, Self::Error> {
        let unbonding_amount = proto
            .unbonding_amount
            .ok_or_else(|| anyhow!("proto malformed"))?
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;
        let balance_blinding_bytes: [u8; 32] = proto.balance_blinding[..]
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;
        let balance_blinding = Fr::from_bytes(balance_blinding_bytes)?;

        Ok(UndelegateClaimProof {
            unbonding_amount,
            balance_blinding,
        })
    }
}
