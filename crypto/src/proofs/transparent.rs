//! Transparent proofs for `MVP1` of the Penumbra system.

use anyhow::{anyhow, ensure, Error, Ok, Result};
use ark_ff::Zero;
use decaf377_rdsa::{SpendAuth, VerificationKey};
use std::convert::{TryFrom, TryInto};

use decaf377::FieldExt;
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
    transaction::Fee,
    Amount, Balance, Fq, Fr, Note, Nullifier, Value,
};

/// Transparent proof for spending existing notes.
///
/// This structure keeps track of the auxiliary (private) inputs.
#[derive(Clone, Debug)]
pub struct SpendProof {
    // Inclusion proof for the note commitment.
    pub state_commitment_proof: tct::Proof,
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
    /// * the merkle root of the state commitment tree,
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

        gadgets::note_commitment_integrity(
            self.note.clone(),
            self.state_commitment_proof.commitment(),
        )?;

        // Merkle path integrity.
        self.state_commitment_proof
            .verify(anchor)
            .map_err(|_| anyhow!("merkle root mismatch"))?;

        let note_balance = Balance::from(self.note.value());

        gadgets::balance_commitment_integrity(balance_commitment, self.v_blinding, note_balance)?;

        gadgets::diversified_basepoint_not_identity(self.note.diversified_generator().clone())?;
        if self.ak.is_identity() {
            return Err(anyhow!("unexpected identity"));
        }

        gadgets::nullifier_integrity(
            nullifier,
            self.nk,
            self.state_commitment_proof.position(),
            self.state_commitment_proof.commitment(),
        )?;

        gadgets::rk_integrity(self.spend_auth_randomizer, rk, self.ak)?;

        gadgets::diversified_address_integrity(self.ak, self.nk, self.note.clone())?;

        Ok(())
    }
}

/// Transparent proof for delegator voting.
///
/// Internally, this is the same data as a transparent spend proof, but with additional verification
/// conditions.
#[derive(Clone, Debug)]
pub struct DelegatorVoteProof {
    pub spend_proof: SpendProof,
}

impl DelegatorVoteProof {
    pub fn verify(
        &self,
        anchor: tct::Root,
        start_position: tct::Position,
        value: Value,
        nullifier: Nullifier,
        rk: VerificationKey<SpendAuth>,
    ) -> anyhow::Result<()> {
        // Additionally, check that the start position has a zero commitment index, since this is
        // the only sensible start time for a vote.
        if start_position.commitment() != 0 {
            return Err(anyhow!(
                "vote proof start position has non-zero commitment index"
            ));
        }

        // Additionally, check that the position of the spend proof is before the start
        // start_height, which ensures that the note being voted with was created before voting
        // started.
        let vote_proof_position = self.spend_proof.state_commitment_proof.position();
        if vote_proof_position >= start_position {
            return Err(anyhow!(
                "vote proof from epoch {}, block {} is not before start position of voting at epoch {}, block {}",
                vote_proof_position.epoch(),
                vote_proof_position.block(),
                start_position.epoch(),
                start_position.block(),
            ));
        }

        // Check that the spend proof is valid, for the public value committed with the zero
        // blinding factor, since it's not blinded.
        self.spend_proof
            .verify(anchor, value.commit(Fr::zero()), nullifier, rk)?;

        Ok(())
    }
}

// Conversions

impl DomainType for SpendProof {
    type Proto = transparent_proofs::SpendProof;
}

impl DomainType for DelegatorVoteProof {
    type Proto = transparent_proofs::SpendProof;
}

impl From<SpendProof> for transparent_proofs::SpendProof {
    fn from(msg: SpendProof) -> Self {
        let ak_bytes: [u8; 32] = msg.ak.into();
        let nk_bytes: [u8; 32] = msg.nk.0.to_bytes();
        transparent_proofs::SpendProof {
            state_commitment_proof: Some(msg.state_commitment_proof.into()),
            note: Some(msg.note.into()),
            v_blinding: msg.v_blinding.to_bytes().to_vec(),
            spend_auth_randomizer: msg.spend_auth_randomizer.to_bytes().to_vec(),
            ak: ak_bytes.into(),
            nk: nk_bytes.into(),
        }
    }
}

impl From<DelegatorVoteProof> for transparent_proofs::SpendProof {
    fn from(msg: DelegatorVoteProof) -> Self {
        msg.spend_proof.into()
    }
}

impl TryFrom<transparent_proofs::SpendProof> for DelegatorVoteProof {
    type Error = Error;

    fn try_from(proto: transparent_proofs::SpendProof) -> anyhow::Result<Self, Self::Error> {
        Ok(DelegatorVoteProof {
            spend_proof: proto.try_into()?,
        })
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
            state_commitment_proof: proto
                .state_commitment_proof
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

impl From<DelegatorVoteProof> for Vec<u8> {
    fn from(delegator_vote_proof: DelegatorVoteProof) -> Vec<u8> {
        let protobuf_serialized_proof: transparent_proofs::SpendProof = delegator_vote_proof.into();
        protobuf_serialized_proof.encode_to_vec()
    }
}

impl TryFrom<&[u8]> for DelegatorVoteProof {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<DelegatorVoteProof, Self::Error> {
        let protobuf_serialized_proof = transparent_proofs::SpendProof::decode(bytes)
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

#[cfg(test)]
mod tests {
    use ark_ff::UniformRand;
    use rand_core::OsRng;

    use super::*;
    use crate::{
        keys::{SeedPhrase, SpendKey},
        Balance, Note, Value,
    };

    #[test]
    /// Check that the `SpendProof` verification succeeds.
    fn test_spend_proof_verification_success() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());
        let v_blinding = Fr::rand(&mut rng);

        let value_to_send = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let spend_auth_randomizer = Fr::rand(&mut rng);
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let mut sct = tct::Tree::new();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();

        let proof = SpendProof {
            state_commitment_proof,
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
    // Check that the `SpendProof` verification fails when using an incorrect
    // TCT root (`anchor`).
    fn test_spend_proof_verification_merkle_path_integrity_failure() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

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
        let mut sct = tct::Tree::new();
        let incorrect_anchor = sct.root();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();

        let proof = SpendProof {
            state_commitment_proof,
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
    /// Check that the `SpendProof` verification fails when using balance
    /// commitments with different blinding factors.
    fn test_spend_proof_verification_balance_commitment_integrity_failure() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let value_to_send = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let balance_to_send = Balance::from(value_to_send);

        let v_blinding = Fr::rand(&mut rng);

        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let spend_auth_randomizer = Fr::rand(&mut rng);

        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();

        let mut sct = tct::Tree::new();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();

        let proof = SpendProof {
            state_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
        };

        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);

        let incorrect_balance_commitment = balance_to_send.commit(Fr::rand(&mut rng));

        assert!(proof
            .verify(anchor, incorrect_balance_commitment, nf, rk)
            .is_err());
    }

    #[test]
    /// Check that the `SpendProof` verification fails, when using an
    /// incorrect nullifier.
    fn test_spend_proof_verification_nullifier_integrity_failure() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

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
        let mut sct = tct::Tree::new();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();

        let proof = SpendProof {
            state_commitment_proof,
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
