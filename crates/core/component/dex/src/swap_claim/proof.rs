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
use penumbra_proto::{core::component::dex::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;
use penumbra_tct::r1cs::StateCommitmentVar;

use penumbra_asset::{
    asset::{self},
    Value, ValueVar,
};
use penumbra_keys::keys::{Bip44Path, NullifierKey, NullifierKeyVar, SeedPhrase, SpendKey};
use penumbra_num::{Amount, AmountVar};
use penumbra_sct::{Nullifier, NullifierVar};
use penumbra_shielded_pool::{
    note::{self, NoteVar},
    Rseed,
};

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

        // Merkle path integrity. Ensure the provided note commitment is in the TCT.
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
        let nullifier = Nullifier(Fq::from(1));
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
        let note_blinding_1 = Fq::from(1);
        let note_blinding_2 = Fq::from(1);
        let note_commitment_1 = tct::StateCommitment(Fq::from(1));
        let note_commitment_2 = tct::StateCommitment(Fq::from(2));
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
    ) -> anyhow::Result<()> {
        let proof =
            Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow::anyhow!(e))?;

        let mut public_inputs = Vec::new();
        public_inputs.extend(
            Fq::from(public.anchor.0)
                .to_field_elements()
                .expect("Fq types are Bls12-377 field members"),
        );
        public_inputs.extend(
            public
                .nullifier
                .0
                .to_field_elements()
                .expect("nullifier is a Bls12-377 field member"),
        );
        public_inputs.extend(
            Fq::from(public.claim_fee.0.amount)
                .to_field_elements()
                .expect("Fq types are Bls12-377 field members"),
        );
        public_inputs.extend(
            public
                .claim_fee
                .0
                .asset_id
                .0
                .to_field_elements()
                .expect("asset_id is a Bls12-377 field member"),
        );
        public_inputs.extend(
            public
                .output_data
                .to_field_elements()
                .expect("output_data is a Bls12-377 field member"),
        );
        public_inputs.extend(
            public
                .note_commitment_1
                .0
                .to_field_elements()
                .expect("note_commitment_1 is a Bls12-377 field member"),
        );
        public_inputs.extend(
            public
                .note_commitment_2
                .0
                .to_field_elements()
                .expect("note_commitment_2 is a Bls12-377 field member"),
        );

        tracing::trace!(?public_inputs);
        let start = std::time::Instant::now();
        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            vk,
            public_inputs.as_slice(),
            &proof,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        tracing::debug!(?proof_result, elapsed = ?start.elapsed());
        proof_result
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("swapclaim proof did not verify"))
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
    use ark_ff::UniformRand;
    use penumbra_keys::keys::{SeedPhrase, SpendKey};
    use penumbra_num::Amount;
    use penumbra_proof_params::generate_prepared_test_parameters;
    use proptest::prelude::*;
    use rand_core::OsRng;

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

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn swap_claim_proof_happy_path_filled(seed_phrase_randomness in any::<[u8; 32]>(), value1_amount in 2..200u64, test_bsod in filled_bsod_strategy()) {
        let mut rng = OsRng;
        let (pk, vk) = generate_prepared_test_parameters::<SwapClaimCircuit>(&mut rng);

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

        let swap_plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1_i, delta_2_i, fee, claim_address);
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

        let public = SwapClaimProofPublic { anchor, nullifier, claim_fee: fee, output_data, note_commitment_1, note_commitment_2 };
        let private = SwapClaimProofPrivate { swap_plaintext, state_commitment_proof, nk, lambda_1, lambda_2, note_blinding_1, note_blinding_2 };

        let blinding_r = Fq::rand(&mut rng);
        let blinding_s = Fq::rand(&mut rng);

        let proof = SwapClaimProof::prove(
            blinding_r,
            blinding_s,
            &pk,
            public.clone(),
            private
        )
        .expect("can create proof");

        let proof_result = proof.verify(&vk, public);

        assert!(proof_result.is_ok());
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

    proptest! {
            #![proptest_config(ProptestConfig::with_cases(2))]
            #[test]
            fn swap_claim_proof_happy_path_unfilled(seed_phrase_randomness in any::<[u8; 32]>(), test_bsod in unfilled_bsod_strategy(), value2_amount in 2..200u64,) {

            let mut rng = OsRng;
            let (pk, vk) = generate_prepared_test_parameters::<SwapClaimCircuit>(&mut rng);

            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_recipient = sk_recipient.full_viewing_key();
            let ivk_recipient = fvk_recipient.incoming();
            let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());
            let nk = *sk_recipient.nullifier_key();

            let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
            let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
            let trading_pair = TradingPair::new(gm.id(), gn.id());

            let delta_1_i = Amount::from(0u64);
            let delta_2_i = Amount::from(value2_amount);
            let fee = Fee::default();

            let swap_plaintext = SwapPlaintext::new(
                &mut rng,
                trading_pair,
                delta_1_i,
                delta_2_i,
                fee,
                claim_address,
            );
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

            let public = SwapClaimProofPublic { anchor, nullifier, claim_fee: fee, output_data, note_commitment_1, note_commitment_2 };
            let private = SwapClaimProofPrivate { swap_plaintext, state_commitment_proof, nk, lambda_1, lambda_2, note_blinding_1, note_blinding_2 };

            let blinding_r = Fq::rand(&mut rng);
            let blinding_s = Fq::rand(&mut rng);

            let proof = SwapClaimProof::prove(
                blinding_r,
                blinding_s,
                &pk,
                public.clone(),
                private
            )
            .expect("can create proof");

            let proof_result = proof.verify(
                &vk,
                public
            );

            assert!(proof_result.is_ok());
        }
    }
}
