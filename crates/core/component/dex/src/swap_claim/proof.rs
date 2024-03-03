use anyhow::Result;
use ark_ff::ToConstraintField;
use ark_groth16::{
    r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, Proof, ProvingKey,
};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use decaf377::{r1cs::FqVar, Bls12_377, Fq};
use penumbra_fee::Fee;
use penumbra_proto::{core::component::dex::v1 as pb, DomainType};
use penumbra_tct as tct;
use penumbra_tct::r1cs::StateCommitmentVar;

use penumbra_asset::{
    asset::{self, Id},
    Value, ValueVar,
};
use penumbra_keys::keys::{Bip44Path, NullifierKey, NullifierKeyVar, SeedPhrase, SpendKey};
use penumbra_num::{Amount, AmountVar};
use penumbra_sct::{Nullifier, NullifierVar};
use penumbra_shielded_pool::{
    note::{self, NoteVar},
    Rseed,
};
use tap::Tap;
use tct::{Root, StateCommitment};

use crate::{
    batch_swap_output_data::BatchSwapOutputDataVar,
    swap::{SwapPlaintext, SwapPlaintextVar},
    BatchSwapOutputData, TradingPair,
};

use penumbra_proof_params::{DummyWitness, GROTH16_PROOF_LENGTH_BYTES};

/// The public inputs to a [`SwapProofPublic`].
#[derive(Clone, Debug)]
pub struct SwapClaimProofPublic {
    /// Anchor
    pub anchor: tct::Root,
    /// Nullifier
    pub nullifier: Nullifier,
    /// Fee
    pub claim_fee: Fee,
    /// Batch swap output data
    pub output_data: BatchSwapOutputData,
    /// Note commitment of first output note
    pub note_commitment_1: note::StateCommitment,
    /// Note commitment of second output note
    pub note_commitment_2: note::StateCommitment,
}

/// The public inputs to a [`SwapProofPrivate`].
#[derive(Clone, Debug)]
pub struct SwapClaimProofPrivate {
    /// The swap being claimed
    pub swap_plaintext: SwapPlaintext,
    /// Inclusion proof for the swap commitment
    pub state_commitment_proof: tct::Proof,
    // The nullifier deriving key for the Swap NFT note.
    pub nk: NullifierKey,
    /// Output amount 1
    pub lambda_1: Amount,
    /// Output amount 2
    pub lambda_2: Amount,
    /// Note commitment blinding factor for the first output note
    pub note_blinding_1: Fq,
    /// Note commitment blinding factor for the second output note
    pub note_blinding_2: Fq,
}

#[cfg(test)]
fn check_satisfaction(
    public: &SwapClaimProofPublic,
    private: &SwapClaimProofPrivate,
) -> Result<()> {
    let swap_commitment = private.swap_plaintext.swap_commitment();
    if swap_commitment != private.state_commitment_proof.commitment() {
        anyhow::bail!("swap commitment integrity check failed");
    }

    private.state_commitment_proof.verify(public.anchor)?;

    let nullifier = Nullifier::derive(
        &private.nk,
        private.state_commitment_proof.position(),
        &swap_commitment,
    );
    if nullifier != public.nullifier {
        anyhow::bail!("nullifier did not match public input");
    }

    if private.swap_plaintext.claim_fee != public.claim_fee {
        anyhow::bail!("claim fee did not match public input");
    }

    let block: u64 = private.state_commitment_proof.position().block().into();
    let note_commitment_block_height: u64 = public.output_data.epoch_starting_height + block;
    if note_commitment_block_height != public.output_data.height {
        anyhow::bail!("swap commitment height did not match public input");
    }

    if private.swap_plaintext.trading_pair != public.output_data.trading_pair {
        anyhow::bail!("trading pair did not match public input");
    }

    let (lambda_1, lambda_2) = public.output_data.pro_rata_outputs((
        private.swap_plaintext.delta_1_i,
        private.swap_plaintext.delta_2_i,
    ));
    if lambda_1 != private.lambda_1 {
        anyhow::bail!("lambda_1 did not match public input");
    }
    if lambda_2 != private.lambda_2 {
        anyhow::bail!("lambda_2 did not match public input");
    }

    let (output_1_note, output_2_note) = private.swap_plaintext.output_notes(&public.output_data);
    let note_commitment_1 = output_1_note.commit();
    let note_commitment_2 = output_2_note.commit();
    if note_commitment_1 != public.note_commitment_1 {
        anyhow::bail!("note commitment 1 did not match public input");
    }
    if note_commitment_2 != public.note_commitment_2 {
        anyhow::bail!("note commitment 2 did not match public input");
    }

    Ok(())
}

#[cfg(test)]
fn check_circuit_satisfaction(
    public: SwapClaimProofPublic,
    private: SwapClaimProofPrivate,
) -> Result<()> {
    use ark_relations::r1cs::{self, ConstraintSystem};

    let cs: ConstraintSystemRef<_> = ConstraintSystem::new_ref();
    let circuit = SwapClaimCircuit { public, private };
    cs.set_optimization_goal(r1cs::OptimizationGoal::Constraints);
    circuit
        .generate_constraints(cs.clone())
        .expect("can generate constraints from circuit");
    cs.finalize();
    if !cs.is_satisfied()? {
        anyhow::bail!("constraints are not satisfied");
    }
    Ok(())
}

/// SwapClaim consumes an existing Swap NFT so they are most similar to Spend operations,
/// however the note commitment proof needs to be for a specific block due to clearing prices
/// only being valid for particular blocks (i.e. the exchange rates of assets change over time).
#[derive(Clone, Debug)]
pub struct SwapClaimCircuit {
    public: SwapClaimProofPublic,
    private: SwapClaimProofPrivate,
}

impl ConstraintSynthesizer<Fq> for SwapClaimCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let swap_plaintext_var =
            SwapPlaintextVar::new_witness(cs.clone(), || Ok(self.private.swap_plaintext.clone()))?;

        let claimed_swap_commitment = StateCommitmentVar::new_witness(cs.clone(), || {
            Ok(self.private.state_commitment_proof.commitment())
        })?;

        let position_var = tct::r1cs::PositionVar::new_witness(cs.clone(), || {
            Ok(self.private.state_commitment_proof.position())
        })?;
        let position_bits = position_var.to_bits_le()?;
        let merkle_path_var = tct::r1cs::MerkleAuthPathVar::new_witness(cs.clone(), || {
            Ok(self.private.state_commitment_proof)
        })?;
        let nk_var = NullifierKeyVar::new_witness(cs.clone(), || Ok(self.private.nk))?;
        let lambda_1_i_var = AmountVar::new_witness(cs.clone(), || Ok(self.private.lambda_1))?;
        let lambda_2_i_var = AmountVar::new_witness(cs.clone(), || Ok(self.private.lambda_2))?;
        let note_blinding_1 = FqVar::new_witness(cs.clone(), || Ok(self.private.note_blinding_1))?;
        let note_blinding_2 = FqVar::new_witness(cs.clone(), || Ok(self.private.note_blinding_2))?;

        // Inputs
        let anchor_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(self.public.anchor)))?;
        let claimed_nullifier_var =
            NullifierVar::new_input(cs.clone(), || Ok(self.public.nullifier))?;
        let claimed_fee_var = ValueVar::new_input(cs.clone(), || Ok(self.public.claim_fee.0))?;
        let output_data_var =
            BatchSwapOutputDataVar::new_input(cs.clone(), || Ok(self.public.output_data))?;
        let claimed_note_commitment_1 =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.public.note_commitment_1))?;
        let claimed_note_commitment_2 =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.public.note_commitment_2))?;

        // Swap commitment integrity check.
        let swap_commitment = swap_plaintext_var.commit()?;
        claimed_swap_commitment.enforce_equal(&swap_commitment)?;

        // Merkle path integrity. Ensure the provided swap commitment is in the TCT.
        merkle_path_var.verify(
            cs.clone(),
            &Boolean::TRUE,
            &position_bits,
            anchor_var,
            claimed_swap_commitment.inner(),
        )?;

        // Nullifier integrity.
        let nullifier_var = NullifierVar::derive(&nk_var, &position_var, &claimed_swap_commitment)?;
        nullifier_var.enforce_equal(&claimed_nullifier_var)?;

        // Fee consistency check.
        claimed_fee_var.enforce_equal(&swap_plaintext_var.claim_fee)?;

        // Validate the swap commitment's height matches the output data's height (i.e. the clearing price height).
        let block = position_var.block()?;
        let note_commitment_block_height_var =
            output_data_var.epoch_starting_height.clone() + block;
        output_data_var
            .height
            .enforce_equal(&note_commitment_block_height_var)?;

        // Validate that the output data's trading pair matches the note commitment's trading pair.
        output_data_var
            .trading_pair
            .enforce_equal(&swap_plaintext_var.trading_pair)?;

        // Output amounts integrity
        let (computed_lambda_1_i, computed_lambda_2_i) = output_data_var.pro_rata_outputs(
            swap_plaintext_var.delta_1_i,
            swap_plaintext_var.delta_2_i,
            cs,
        )?;
        computed_lambda_1_i.enforce_equal(&lambda_1_i_var)?;
        computed_lambda_2_i.enforce_equal(&lambda_2_i_var)?;

        // Output note integrity
        let output_1_note = NoteVar {
            address: swap_plaintext_var.claim_address.clone(),
            value: ValueVar {
                amount: lambda_1_i_var,
                asset_id: swap_plaintext_var.trading_pair.asset_1,
            },
            note_blinding: note_blinding_1,
        };
        let output_1_commitment = output_1_note.commit()?;
        let output_2_note = NoteVar {
            address: swap_plaintext_var.claim_address,
            value: ValueVar {
                amount: lambda_2_i_var,
                asset_id: swap_plaintext_var.trading_pair.asset_2,
            },
            note_blinding: note_blinding_2,
        };
        let output_2_commitment = output_2_note.commit()?;

        claimed_note_commitment_1.enforce_equal(&output_1_commitment)?;
        claimed_note_commitment_2.enforce_equal(&output_2_commitment)?;

        Ok(())
    }
}

impl DummyWitness for SwapClaimCircuit {
    fn with_dummy_witness() -> Self {
        let trading_pair = TradingPair {
            asset_1: asset::Cache::with_known_assets()
                .get_unit("upenumbra")
                .expect("upenumbra denom is known")
                .id(),
            asset_2: asset::Cache::with_known_assets()
                .get_unit("nala")
                .expect("nala denom is known")
                .id(),
        };

        let seed_phrase = SeedPhrase::from_randomness(&[b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());
        let nk = *sk_sender.nullifier_key();

        let delta_1_i = 10u64.into();
        let delta_2_i = 1u64.into();
        let swap_plaintext = SwapPlaintext {
            trading_pair,
            delta_1_i,
            delta_2_i,
            claim_fee: Fee(Value {
                amount: 3u64.into(),
                asset_id: asset::Cache::with_known_assets()
                    .get_unit("upenumbra")
                    .expect("upenumbra denom is known")
                    .id(),
            }),
            claim_address: address,
            rseed: Rseed([1u8; 32]),
        };
        let mut sct = tct::Tree::new();
        let swap_commitment = swap_plaintext.swap_commitment();
        sct.insert(tct::Witness::Keep, swap_commitment)
            .expect("insertion of the swap commitment into the SCT should succeed");
        let anchor = sct.root();
        let state_commitment_proof = sct
            .witness(swap_commitment)
            .expect("the SCT should be able to witness the just-inserted swap commitment");
        let nullifier = Nullifier(Fq::from(1u64));
        let claim_fee = Fee::default();
        let output_data = BatchSwapOutputData {
            delta_1: Amount::from(10u64),
            delta_2: Amount::from(10u64),
            lambda_1: Amount::from(10u64),
            lambda_2: Amount::from(10u64),
            unfilled_1: Amount::from(10u64),
            unfilled_2: Amount::from(10u64),
            height: 0,
            trading_pair: swap_plaintext.trading_pair,
            epoch_starting_height: 0,
        };
        let note_blinding_1 = Fq::from(1u64);
        let note_blinding_2 = Fq::from(1u64);
        let note_commitment_1 = tct::StateCommitment(Fq::from(1u64));
        let note_commitment_2 = tct::StateCommitment(Fq::from(2u64));
        let (lambda_1, lambda_2) = output_data.pro_rata_outputs((delta_1_i, delta_2_i));

        let public = SwapClaimProofPublic {
            anchor,
            nullifier,
            claim_fee,
            output_data,
            note_commitment_1,
            note_commitment_2,
        };
        let private = SwapClaimProofPrivate {
            swap_plaintext,
            state_commitment_proof,
            nk,
            lambda_1,
            lambda_2,
            note_blinding_1,
            note_blinding_2,
        };

        Self { public, private }
    }
}

#[derive(Clone, Debug)]
pub struct SwapClaimProof(pub [u8; GROTH16_PROOF_LENGTH_BYTES]);

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("error deserializing compressed proof: {0:?}")]
    ProofDeserialize(ark_serialize::SerializationError),
    #[error("Fq types are Bls12-377 field members")]
    Anchor,
    #[error("nullifier is a Bls12-377 field member")]
    Nullifier,
    #[error("Fq types are Bls12-377 field members")]
    ClaimFeeAmount,
    #[error("asset_id is a Bls12-377 field member")]
    ClaimFeeAssetId,
    #[error("output_data is a Bls12-377 field member")]
    OutputData,
    #[error("note_commitment_1 is a Bls12-377 field member")]
    NoteCommitment1,
    #[error("note_commitment_2 is a Bls12-377 field member")]
    NoteCommitment2,
    #[error("error verifying proof: {0:?}")]
    SynthesisError(ark_relations::r1cs::SynthesisError),
    #[error("proof did not verify")]
    InvalidProof,
}

impl SwapClaimProof {
    #![allow(clippy::too_many_arguments)]
    /// Generate an [`SwapClaimProof`] given the proving key, public inputs,
    /// witness data, and two random elements `blinding_r` and `blinding_s`.
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        public: SwapClaimProofPublic,
        private: SwapClaimProofPrivate,
    ) -> anyhow::Result<Self> {
        let circuit = SwapClaimCircuit { public, private };

        let proof = Groth16::<Bls12_377, LibsnarkReduction>::create_proof_with_reduction(
            circuit, pk, blinding_r, blinding_s,
        )
        .map_err(|err| anyhow::anyhow!(err))?;

        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof, &mut proof_bytes[..]).expect("can serialize Proof");
        Ok(Self(proof_bytes))
    }

    /// Called to verify the proof using the provided public inputs.
    //#[tracing::instrument(skip(self, vk), fields(self = ?base64::encode(&self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    #[tracing::instrument(skip(self, vk))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        public: SwapClaimProofPublic,
    ) -> Result<(), VerificationError> {
        let proof = Proof::deserialize_compressed_unchecked(&self.0[..])
            .map_err(VerificationError::ProofDeserialize)?;

        let mut public_inputs = Vec::new();

        let SwapClaimProofPublic {
            anchor: Root(anchor),
            nullifier: Nullifier(nullifier),
            claim_fee:
                Fee(Value {
                    amount,
                    asset_id: Id(asset_id),
                }),
            output_data,
            note_commitment_1: StateCommitment(note_commitment_1),
            note_commitment_2: StateCommitment(note_commitment_2),
        } = public;

        public_inputs.extend(
            Fq::from(anchor)
                .to_field_elements()
                .ok_or(VerificationError::Anchor)?,
        );
        public_inputs.extend(
            nullifier
                .to_field_elements()
                .ok_or(VerificationError::Nullifier)?,
        );
        public_inputs.extend(
            Fq::from(amount)
                .to_field_elements()
                .ok_or(VerificationError::ClaimFeeAmount)?,
        );
        public_inputs.extend(
            asset_id
                .to_field_elements()
                .ok_or(VerificationError::ClaimFeeAssetId)?,
        );
        public_inputs.extend(
            output_data
                .to_field_elements()
                .ok_or(VerificationError::OutputData)?,
        );
        public_inputs.extend(
            note_commitment_1
                .to_field_elements()
                .ok_or(VerificationError::NoteCommitment1)?,
        );
        public_inputs.extend(
            note_commitment_2
                .to_field_elements()
                .ok_or(VerificationError::NoteCommitment2)?,
        );

        tracing::trace!(?public_inputs);
        let start = std::time::Instant::now();
        Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            vk,
            public_inputs.as_slice(),
            &proof,
        )
        .map_err(VerificationError::SynthesisError)?
        .tap(|proof_result| tracing::debug!(?proof_result, elapsed = ?start.elapsed()))
        .then_some(())
        .ok_or(VerificationError::InvalidProof)
    }
}

impl DomainType for SwapClaimProof {
    type Proto = pb::ZkSwapClaimProof;
}

impl From<SwapClaimProof> for pb::ZkSwapClaimProof {
    fn from(proof: SwapClaimProof) -> Self {
        pb::ZkSwapClaimProof {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkSwapClaimProof> for SwapClaimProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkSwapClaimProof) -> Result<Self, Self::Error> {
        Ok(SwapClaimProof(proto.inner[..].try_into()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use penumbra_keys::keys::{SeedPhrase, SpendKey};
    use penumbra_num::Amount;
    use proptest::prelude::*;

    #[derive(Debug)]
    struct TestBatchSwapOutputData {
        delta_1: Amount,
        delta_2: Amount,
        lambda_1: Amount,
        lambda_2: Amount,
        unfilled_1: Amount,
        unfilled_2: Amount,
    }

    fn filled_bsod_strategy() -> BoxedStrategy<TestBatchSwapOutputData> {
        let delta_1 = (4001..2000000000u128).prop_map(Amount::from);
        let delta_2 = (4001..2000000000u128).prop_map(Amount::from);

        let lambda_1 = (2..2000u64).prop_map(Amount::from);
        let lambda_2 = (2..2000u64).prop_map(Amount::from);

        let unfilled_1 = (2..2000u64).prop_map(Amount::from);
        let unfilled_2 = (2..2000u64).prop_map(Amount::from);

        (delta_1, delta_2, lambda_1, lambda_2, unfilled_1, unfilled_2)
            .prop_flat_map(
                move |(delta_1, delta_2, lambda_1, lambda_2, unfilled_1, unfilled_2)| {
                    (
                        Just(delta_1),
                        Just(delta_2),
                        Just(lambda_1),
                        Just(lambda_2),
                        Just(unfilled_1),
                        Just(unfilled_2),
                    )
                },
            )
            .prop_map(
                move |(delta_1, delta_2, lambda_1, lambda_2, unfilled_1, unfilled_2)| {
                    TestBatchSwapOutputData {
                        delta_1,
                        delta_2,
                        lambda_1,
                        lambda_2,
                        unfilled_1,
                        unfilled_2,
                    }
                },
            )
            .boxed()
    }

    fn swapclaim_statement(
        seed_phrase_randomness: [u8; 32],
        rseed_randomness: [u8; 32],
        value1_amount: u64,
        test_bsod: TestBatchSwapOutputData,
    ) -> (SwapClaimProofPublic, SwapClaimProofPrivate) {
        let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
        let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());
        let nk = *sk_recipient.nullifier_key();

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
        let trading_pair = TradingPair::new(gm.id(), gn.id());

        let delta_1_i = Amount::from(value1_amount);
        let delta_2_i = Amount::from(0u64);
        let fee = Fee::default();

        let rseed = Rseed(rseed_randomness);
        let swap_plaintext = SwapPlaintext {
            trading_pair,
            delta_1_i,
            delta_2_i,
            claim_fee: fee,
            claim_address,
            rseed,
        };
        let fee = swap_plaintext.clone().claim_fee;
        let mut sct = tct::Tree::new();
        let swap_commitment = swap_plaintext.swap_commitment();
        sct.insert(tct::Witness::Keep, swap_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(swap_commitment).unwrap();
        let position = state_commitment_proof.position();
        let nullifier = Nullifier::derive(&nk, position, &swap_commitment);
        let epoch_duration = 20;
        let height = epoch_duration * position.epoch() + position.block();

        let output_data = BatchSwapOutputData {
            delta_1: test_bsod.delta_1,
            delta_2: test_bsod.delta_2,
            lambda_1: test_bsod.lambda_1,
            lambda_2: test_bsod.lambda_2,
            unfilled_1: test_bsod.unfilled_1,
            unfilled_2: test_bsod.unfilled_2,
            height: height.into(),
            trading_pair: swap_plaintext.trading_pair,
            epoch_starting_height: (epoch_duration * position.epoch()).into(),
        };
        let (lambda_1, lambda_2) = output_data.pro_rata_outputs((delta_1_i, delta_2_i));

        let (output_rseed_1, output_rseed_2) = swap_plaintext.output_rseeds();
        let note_blinding_1 = output_rseed_1.derive_note_blinding();
        let note_blinding_2 = output_rseed_2.derive_note_blinding();
        let (output_1_note, output_2_note) = swap_plaintext.output_notes(&output_data);
        let note_commitment_1 = output_1_note.commit();
        let note_commitment_2 = output_2_note.commit();

        let public = SwapClaimProofPublic {
            anchor,
            nullifier,
            claim_fee: fee,
            output_data,
            note_commitment_1,
            note_commitment_2,
        };
        let private = SwapClaimProofPrivate {
            swap_plaintext,
            state_commitment_proof,
            nk,
            lambda_1,
            lambda_2,
            note_blinding_1,
            note_blinding_2,
        };

        (public, private)
    }

    prop_compose! {
        fn arb_valid_swapclaim_statement_filled()(seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>(), value1_amount in 2..200u64, test_bsod in filled_bsod_strategy()) -> (SwapClaimProofPublic, SwapClaimProofPrivate) {
            swapclaim_statement(seed_phrase_randomness, rseed_randomness, value1_amount, test_bsod)
        }
    }

    proptest! {
        #[test]
        fn swap_claim_proof_happy_path_filled((public, private) in arb_valid_swapclaim_statement_filled()) {
            assert!(check_satisfaction(&public, &private).is_ok());
            assert!(check_circuit_satisfaction(public, private).is_ok());
        }
    }

    fn unfilled_bsod_strategy() -> BoxedStrategy<TestBatchSwapOutputData> {
        let delta_1: Amount = 0u64.into();
        let delta_2 = (4001..2000000000u128).prop_map(Amount::from);

        let lambda_1: Amount = 0u64.into();
        let lambda_2: Amount = 0u64.into();

        let unfilled_1: Amount = 0u64.into();
        let unfilled_2 = delta_2.clone();

        (delta_2, unfilled_2)
            .prop_flat_map(move |(delta_2, unfilled_2)| (Just(delta_2), Just(unfilled_2)))
            .prop_map(move |(delta_2, unfilled_2)| TestBatchSwapOutputData {
                delta_1,
                delta_2,
                lambda_1,
                lambda_2,
                unfilled_1,
                unfilled_2,
            })
            .boxed()
    }

    prop_compose! {
        fn arb_valid_swapclaim_statement_unfilled()(seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>(), value1_amount in 2..200u64, test_bsod in unfilled_bsod_strategy()) -> (SwapClaimProofPublic, SwapClaimProofPrivate) {
            swapclaim_statement(seed_phrase_randomness, rseed_randomness, value1_amount, test_bsod)
        }
    }

    proptest! {
        #[test]
        fn swap_claim_proof_happy_path_unfilled((public, private) in arb_valid_swapclaim_statement_unfilled()) {
            assert!(check_satisfaction(&public, &private).is_ok());
            assert!(check_circuit_satisfaction(public, private).is_ok());
        }
    }

    prop_compose! {
        // This strategy is invalid because the fee is not equal to the claim fee.
        fn arb_invalid_swapclaim_statement_fee()(seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>(), value1_amount in 2..200u64, fee_amount in any::<u64>(), test_bsod in unfilled_bsod_strategy()) -> (SwapClaimProofPublic, SwapClaimProofPrivate) {
            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
        let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());
        let nk = *sk_recipient.nullifier_key();

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
        let trading_pair = TradingPair::new(gm.id(), gn.id());

        let delta_1_i = Amount::from(value1_amount);
        let delta_2_i = Amount::from(0u64);
        let fee = Fee::default();

        let rseed = Rseed(rseed_randomness);
        let swap_plaintext = SwapPlaintext {
            trading_pair,
            delta_1_i,
            delta_2_i,
            claim_fee: fee,
            claim_address,
            rseed,
        };
        let incorrect_fee = Fee::from_staking_token_amount(Amount::from(fee_amount));
        let mut sct = tct::Tree::new();
        let swap_commitment = swap_plaintext.swap_commitment();
        sct.insert(tct::Witness::Keep, swap_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(swap_commitment).unwrap();
        let position = state_commitment_proof.position();
        let nullifier = Nullifier::derive(&nk, position, &swap_commitment);
        let epoch_duration = 20;
        let height = epoch_duration * position.epoch() + position.block();

        let output_data = BatchSwapOutputData {
            delta_1: test_bsod.delta_1,
            delta_2: test_bsod.delta_2,
            lambda_1: test_bsod.lambda_1,
            lambda_2: test_bsod.lambda_2,
            unfilled_1: test_bsod.unfilled_1,
            unfilled_2: test_bsod.unfilled_2,
            height: height.into(),
            trading_pair: swap_plaintext.trading_pair,
            epoch_starting_height: (epoch_duration * position.epoch()).into(),
        };
        let (lambda_1, lambda_2) = output_data.pro_rata_outputs((delta_1_i, delta_2_i));

        let (output_rseed_1, output_rseed_2) = swap_plaintext.output_rseeds();
        let note_blinding_1 = output_rseed_1.derive_note_blinding();
        let note_blinding_2 = output_rseed_2.derive_note_blinding();
        let (output_1_note, output_2_note) = swap_plaintext.output_notes(&output_data);
        let note_commitment_1 = output_1_note.commit();
        let note_commitment_2 = output_2_note.commit();

        let public = SwapClaimProofPublic {
            anchor,
            nullifier,
            claim_fee: incorrect_fee,
            output_data,
            note_commitment_1,
            note_commitment_2,
        };
        let private = SwapClaimProofPrivate {
            swap_plaintext,
            state_commitment_proof,
            nk,
            lambda_1,
            lambda_2,
            note_blinding_1,
            note_blinding_2,
        };

        (public, private)
        }
    }

    proptest! {
        #[test]
        fn swap_claim_proof_invalid_fee((public, private) in arb_invalid_swapclaim_statement_fee()) {
            assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
        }
    }

    prop_compose! {
        // This strategy is invalid because the block height of the swap commitment does not match
        // the height of the batch swap output data.
        fn arb_invalid_swapclaim_swap_commitment_height()(seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>(), value1_amount in 2..200u64, fee_amount in any::<u64>(), test_bsod in unfilled_bsod_strategy()) -> (SwapClaimProofPublic, SwapClaimProofPrivate) {
            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
        let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());
        let nk = *sk_recipient.nullifier_key();

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
        let trading_pair = TradingPair::new(gm.id(), gn.id());

        let delta_1_i = Amount::from(value1_amount);
        let delta_2_i = Amount::from(0u64);
        let fee = Fee::default();

        let rseed = Rseed(rseed_randomness);
        let swap_plaintext = SwapPlaintext {
            trading_pair,
            delta_1_i,
            delta_2_i,
            claim_fee: fee,
            claim_address,
            rseed,
        };
        let incorrect_fee = Fee::from_staking_token_amount(Amount::from(fee_amount));
        let mut sct = tct::Tree::new();
        let swap_commitment = swap_plaintext.swap_commitment();
        sct.insert(tct::Witness::Keep, swap_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(swap_commitment).unwrap();
        let position = state_commitment_proof.position();
        let nullifier = Nullifier::derive(&nk, position, &swap_commitment);

        // End the block, and then add a dummy commitment that we'll use
        // to compute the position and block height that the BSOD corresponds to.
        sct.end_block().expect("can end block");
        let dummy_swap_commitment = tct::StateCommitment(Fq::from(1u64));
        sct.insert(tct::Witness::Keep, dummy_swap_commitment).unwrap();
        let dummy_state_commitment_proof = sct.witness(swap_commitment).unwrap();
        let dummy_position = dummy_state_commitment_proof.position();

        let epoch_duration = 20;
        let height = epoch_duration * dummy_position.epoch() + dummy_position.block();

        let output_data = BatchSwapOutputData {
            delta_1: test_bsod.delta_1,
            delta_2: test_bsod.delta_2,
            lambda_1: test_bsod.lambda_1,
            lambda_2: test_bsod.lambda_2,
            unfilled_1: test_bsod.unfilled_1,
            unfilled_2: test_bsod.unfilled_2,
            height: height.into(),
            trading_pair: swap_plaintext.trading_pair,
            epoch_starting_height: (epoch_duration * dummy_position.epoch()).into(),
        };
        let (lambda_1, lambda_2) = output_data.pro_rata_outputs((delta_1_i, delta_2_i));

        let (output_rseed_1, output_rseed_2) = swap_plaintext.output_rseeds();
        let note_blinding_1 = output_rseed_1.derive_note_blinding();
        let note_blinding_2 = output_rseed_2.derive_note_blinding();
        let (output_1_note, output_2_note) = swap_plaintext.output_notes(&output_data);
        let note_commitment_1 = output_1_note.commit();
        let note_commitment_2 = output_2_note.commit();

        let public = SwapClaimProofPublic {
            anchor,
            nullifier,
            claim_fee: incorrect_fee,
            output_data,
            note_commitment_1,
            note_commitment_2,
        };
        let private = SwapClaimProofPrivate {
            swap_plaintext,
            state_commitment_proof,
            nk,
            lambda_1,
            lambda_2,
            note_blinding_1,
            note_blinding_2,
        };

        (public, private)
        }
    }

    proptest! {
        #[test]
        fn swap_claim_proof_invalid_swap_commitment_height((public, private) in arb_invalid_swapclaim_swap_commitment_height()) {
            assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
        }
    }
}
