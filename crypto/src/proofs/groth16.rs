mod gadgets;
mod output;
mod spend;
mod swap;
mod traits;

pub use output::{OutputCircuit, OutputProof};
pub use spend::{SpendCircuit, SpendProof};
pub use swap::{SwapCircuit, SwapProof};
pub use traits::{ParameterSetup, VerifyingKeyExt};

/// The length of our Groth16 proofs in bytes.
pub const GROTH16_PROOF_LENGTH_BYTES: usize = 192;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{
        asset,
        dex::{swap::SwapPlaintext, TradingPair},
        keys::{SeedPhrase, SpendKey},
        transaction::Fee,
        Address, Amount, Balance, Rseed,
    };
    use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
    use ark_r1cs_std::prelude::*;
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
    use ark_snark::SNARK;
    use decaf377::{r1cs::FqVar, Bls12_377, Fq, Fr};
    use proptest::prelude::*;

    use decaf377_rdsa::{SpendAuth, VerificationKey};
    use penumbra_proto::core::crypto::v1alpha1 as pb;
    use penumbra_tct as tct;
    use rand_core::OsRng;
    use tct::Commitment;

    use crate::{note, Note, Value};

    use ark_ff::PrimeField;

    fn fq_strategy() -> BoxedStrategy<Fq> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fq::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    fn fr_strategy() -> BoxedStrategy<Fr> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fr::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn swap_proof_happy_path(seed_phrase_randomness in any::<[u8; 32]>(), fee_blinding in fr_strategy(), value1_amount in 2..200u64) {
        let (pk, vk) = SwapCircuit::generate_prepared_test_parameters();

        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());

        let gm = asset::REGISTRY.parse_unit("gm");
        let gn = asset::REGISTRY.parse_unit("gn");
        let trading_pair = TradingPair::new(gm.id(), gn.id());

        let delta_1 = Amount::from(value1_amount);
        let delta_2 = Amount::from(0u64);
        let fee = Fee::default();

        let swap_plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1, delta_2, fee, claim_address);
        let fee_commitment = swap_plaintext.claim_fee.commit(fee_blinding);
        let swap_commitment = swap_plaintext.swap_commitment();

        let value_1 = Value {
            amount: swap_plaintext.delta_1_i,
            asset_id: swap_plaintext.trading_pair.asset_1(),
        };
        let value_2 = Value {
            amount: swap_plaintext.delta_2_i,
            asset_id:  swap_plaintext.trading_pair.asset_2(),
        };
        let value_fee = Value {
            amount: swap_plaintext.claim_fee.amount(),
            asset_id: swap_plaintext.claim_fee.asset_id(),
        };
        let mut balance = Balance::default();
        balance -= value_1;
        balance -= value_2;
        balance -= value_fee;
        let balance_commitment = balance.commit(fee_blinding);

            let proof = SwapProof::prove(
                &mut rng,
                &pk,
                swap_plaintext,
                fee_blinding,
                balance_commitment,
                swap_commitment,
                fee_commitment
            )
            .expect("can create proof");

            let proof_result = proof.verify(&vk, balance_commitment, swap_commitment, fee_commitment);

            assert!(proof_result.is_ok());
        }
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn output_proof_happy_path(seed_phrase_randomness in any::<[u8; 32]>(), v_blinding in fr_strategy(), value_amount in 2..200u64) {
            let (pk, vk) = OutputCircuit::generate_prepared_test_parameters();

            let mut rng = OsRng;

            let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
            let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
            let fvk_recipient = sk_recipient.full_viewing_key();
            let ivk_recipient = fvk_recipient.incoming();
            let (dest, _dtk_d) = ivk_recipient.payment_address(0u32.into());

            let value_to_send = Value {
                amount: value_amount.into(),
                asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &dest, value_to_send);
            let note_commitment = note.commit();
            let balance_commitment = (-Balance::from(value_to_send)).commit(v_blinding);

            let proof = OutputProof::prove(
                &mut rng,
                &pk,
                note,
                v_blinding,
                balance_commitment,
                note_commitment,
            )
            .expect("can create proof");
            let serialized_proof: pb::ZkOutputProof = proof.into();

            let deserialized_proof = OutputProof::try_from(serialized_proof).expect("can deserialize proof");
            let proof_result = deserialized_proof.verify(&vk, balance_commitment, note_commitment);

            assert!(proof_result.is_ok());
        }
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn output_proof_verification_note_commitment_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), v_blinding in fr_strategy(), value_amount in 2..200u64, note_blinding in fq_strategy()) {
        let (pk, vk) = OutputCircuit::generate_prepared_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u32.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &dest, value_to_send);
        let note_commitment = note.commit();
        let balance_commitment = (-Balance::from(value_to_send)).commit(v_blinding);

        let proof = OutputProof::prove(
            &mut rng,
            &pk,
            note.clone(),
            v_blinding,
            balance_commitment,
            note_commitment,
        )
        .expect("can create proof");

        let incorrect_note_commitment = note::commitment(
            note_blinding,
            value_to_send,
            note.diversified_generator(),
            note.transmission_key_s(),
            note.clue_key(),
        );

        let proof_result = proof.verify(&vk, balance_commitment, incorrect_note_commitment);

        assert!(proof_result.is_err());
    }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2))]
        #[test]
    fn output_proof_verification_balance_commitment_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), v_blinding in fr_strategy(), value_amount in 2..200u64, incorrect_v_blinding in fr_strategy()) {
        let (pk, vk) = OutputCircuit::generate_prepared_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u32.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &dest, value_to_send);
        let note_commitment = note.commit();
        let balance_commitment = (-Balance::from(value_to_send)).commit(v_blinding);

        let proof = OutputProof::prove(
            &mut rng,
            &pk,
            note,
            v_blinding,
            balance_commitment,
            note_commitment,
        )
        .expect("can create proof");

        let incorrect_balance_commitment = (-Balance::from(value_to_send)).commit(incorrect_v_blinding);

        let proof_result = proof.verify(&vk, incorrect_balance_commitment, note_commitment);

        assert!(proof_result.is_err());
    }
        }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    /// Check that the `SpendProof` verification succeeds.
    fn spend_proof_verification_success(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 2..2000000000u64, num_commitments in 1..2000u64, v_blinding in fr_strategy()) {
        let (pk, vk) = SpendCircuit::generate_prepared_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
        let mut sct = tct::Tree::new();

        // Next, we simulate the case where the SCT is not empty by adding `num_commitments`
        // unrelated items in the SCT.
        for _ in 0..num_commitments {
            let random_note_commitment = Note::generate(&mut rng, &sender, value_to_send).commit();
            sct.insert(tct::Witness::Keep, random_note_commitment).unwrap();
        }

        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(state_commitment_proof.position(), &note_commitment);

        let proof = SpendProof::prove(
            &mut rng,
            &pk,
            state_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment,
            nf,
            rk,
        )
        .expect("can create proof");

        let proof_result = proof.verify(&vk, anchor, balance_commitment, nf, rk);
        assert!(proof_result.is_ok());
    }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    /// Check that the `SpendProof` verification fails when using an incorrect
    /// TCT root (`anchor`).
    fn spend_proof_verification_merkle_path_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 2..200u64, v_blinding in fr_strategy()) {
        let (pk, vk) = SpendCircuit::generate_prepared_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
        let mut sct = tct::Tree::new();
        let incorrect_anchor = sct.root();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);

        let proof = SpendProof::prove(
            &mut rng,
            &pk,
            state_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment,
            nf,
            rk,
        )
        .expect("can create proof");

        let proof_result = proof.verify(&vk, incorrect_anchor, balance_commitment, nf, rk);
        assert!(proof_result.is_err());
    }
    }

    proptest! {
            #![proptest_config(ProptestConfig::with_cases(2))]
            #[should_panic]
        #[test]
        /// Check that the `SpendProof` verification fails when the diversified address is wrong.
        fn spend_proof_verification_diversified_address_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), incorrect_seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 2..200u64, v_blinding in fr_strategy()) {
            let (pk, vk) = SpendCircuit::generate_prepared_test_parameters();
            let mut rng = OsRng;

            let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);

            let wrong_seed_phrase = SeedPhrase::from_randomness(incorrect_seed_phrase_randomness);
            let wrong_sk_sender = SpendKey::from_seed_phrase(wrong_seed_phrase, 0);
            let wrong_fvk_sender = wrong_sk_sender.full_viewing_key();
            let wrong_ivk_sender = wrong_fvk_sender.incoming();
            let (wrong_sender, _dtk_d) = wrong_ivk_sender.payment_address(1u32.into());

            let value_to_send = Value {
                amount: value_amount.into(),
                asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &wrong_sender, value_to_send);

            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak = sk_sender.spend_auth_key().into();
            let mut sct = tct::Tree::new();
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nf = nk.derive_nullifier(0.into(), &note_commitment);

            // Note that this will blow up in debug mode as the constraint
            // system is unsatisified (ark-groth16 has a debug check for this).
            // In release mode the proof will be created, but will fail to verify.
            let proof = SpendProof::prove(
                &mut rng,
                &pk,
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                anchor,
                balance_commitment,
                nf,
                rk,
            ).expect("can create proof in release mode");

            proof.verify(&vk, anchor, balance_commitment, nf, rk).expect("boom");
        }
    }

    proptest! {
            #![proptest_config(ProptestConfig::with_cases(2))]
        #[test]
        /// Check that the `SpendProof` verification fails, when using an
        /// incorrect nullifier.
        fn spend_proof_verification_nullifier_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 2..200u64, v_blinding in fr_strategy()) {
            let (pk, vk) = SpendCircuit::generate_prepared_test_parameters();
            let mut rng = OsRng;

            let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

            let value_to_send = Value {
                amount: value_amount.into(),
                asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &sender, value_to_send);
            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak = sk_sender.spend_auth_key().into();
            let mut sct = tct::Tree::new();
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nf = nk.derive_nullifier(0.into(), &note_commitment);

            let incorrect_nf = nk.derive_nullifier(5.into(), &note_commitment);

            let proof = SpendProof::prove(
                &mut rng,
                &pk,
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                anchor,
                balance_commitment,
                nf,
                rk,
            )
            .expect("can create proof");

            let proof_result = proof.verify(&vk, anchor, balance_commitment, incorrect_nf, rk);
            assert!(proof_result.is_err());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    /// Check that the `SpendProof` verification fails when using balance
    /// commitments with different blinding factors.
    fn spend_proof_verification_balance_commitment_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 2..200u64, v_blinding in fr_strategy(), incorrect_blinding_factor in fr_strategy()) {
        let (pk, vk) = SpendCircuit::generate_prepared_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let mut sct = tct::Tree::new();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);

        let proof = SpendProof::prove(
            &mut rng,
            &pk,
            state_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment,
            nf,
            rk,
        )
        .expect("can create proof");

        let incorrect_balance_commitment = value_to_send.commit(incorrect_blinding_factor);

        let proof_result = proof.verify(&vk, anchor, incorrect_balance_commitment, nf, rk);
        assert!(proof_result.is_err());
    }
    }

    proptest! {
            #![proptest_config(ProptestConfig::with_cases(2))]
        #[test]
        /// Check that the `SpendProof` verification fails when the incorrect randomizable verification key is used.
        fn spend_proof_verification_fails_rk_integrity(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 2..200u64, v_blinding in fr_strategy(), incorrect_spend_auth_randomizer in fr_strategy()) {
            let (pk, vk) = SpendCircuit::generate_prepared_test_parameters();
            let mut rng = OsRng;

            let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

            let value_to_send = Value {
                amount: value_amount.into(),
                asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &sender, value_to_send);
            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak = sk_sender.spend_auth_key().into();
            let mut sct = tct::Tree::new();
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nf = nk.derive_nullifier(0.into(), &note_commitment);

            let incorrect_rsk = sk_sender
                .spend_auth_key()
                .randomize(&incorrect_spend_auth_randomizer);
            let incorrect_rk: VerificationKey<SpendAuth> = incorrect_rsk.into();

            let proof = SpendProof::prove(
                &mut rng,
                &pk,
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                anchor,
                balance_commitment,
                nf,
                rk,
            )
            .expect("should be able to form proof");

            let proof_result = proof.verify(&vk, anchor, balance_commitment, nf, incorrect_rk);
            assert!(proof_result.is_err());
        }
    }

    proptest! {
            #![proptest_config(ProptestConfig::with_cases(2))]
        #[test]
        /// Check that the `SpendProof` verification always suceeds for dummy (zero value) spends.
        fn spend_proof_dummy_verification_suceeds(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), v_blinding in fr_strategy()) {
            let (pk, vk) = SpendCircuit::generate_prepared_test_parameters();
            let mut rng = OsRng;

            let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

            let value_to_send = Value {
                amount: 0u64.into(),
                asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &sender, value_to_send);
            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak = sk_sender.spend_auth_key().into();
            let mut sct = tct::Tree::new();
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            // Using a random blinding factor here, but the proof will verify
            // since for dummies we only check if the value is zero, and choose
            // not to enforce the other equality constraint.
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nf = nk.derive_nullifier(0.into(), &note_commitment);

            let proof = SpendProof::prove(
                &mut rng,
                &pk,
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                anchor,
                balance_commitment,
                nf,
                rk,
            )
            .expect("should be able to form proof");

            let proof_result = proof.verify(&vk, anchor, balance_commitment, nf, rk);
            assert!(proof_result.is_ok());
        }
    }

    struct MerkleProofCircuit {
        /// Witness: Inclusion proof for the note commitment.
        state_commitment_proof: tct::Proof,
        /// Public input: The merkle root of the state commitment tree
        pub anchor: tct::Root,
    }

    impl ConstraintSynthesizer<Fq> for MerkleProofCircuit {
        fn generate_constraints(
            self,
            cs: ConstraintSystemRef<Fq>,
        ) -> ark_relations::r1cs::Result<()> {
            let merkle_path_var =
                tct::r1cs::MerkleAuthPathVar::new(cs.clone(), self.state_commitment_proof.clone())?;
            let anchor_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(self.anchor)))?;
            let claimed_note_commitment =
                note::StateCommitmentVar::new_witness(cs.clone(), || {
                    Ok(self.state_commitment_proof.commitment())
                })?;
            let position_var = tct::r1cs::PositionVar::new_witness(cs.clone(), || {
                Ok(self.state_commitment_proof.position())
            })?;
            merkle_path_var.verify(
                cs.clone(),
                &Boolean::TRUE,
                position_var.inner,
                anchor_var,
                claimed_note_commitment.inner(),
            )?;
            Ok(())
        }
    }

    impl ParameterSetup for MerkleProofCircuit {
        fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
            let seed_phrase = SeedPhrase::from_randomness([b'f'; 32]);
            let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());

            let note = Note::from_parts(
                address,
                Value::from_str("1upenumbra").expect("valid value"),
                Rseed([1u8; 32]),
            )
            .expect("can make a note");
            let mut sct = tct::Tree::new();
            let note_commitment = note.commit();
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let circuit = MerkleProofCircuit {
                state_commitment_proof,
                anchor,
            };
            let (pk, vk) = Groth16::circuit_specific_setup(circuit, &mut OsRng)
                .expect("can perform circuit specific setup");
            (pk, vk)
        }
    }

    fn make_random_note_commitment(address: Address) -> Commitment {
        let note = Note::from_parts(
            address,
            Value::from_str("1upenumbra").expect("valid value"),
            Rseed([1u8; 32]),
        )
        .expect("can make a note");
        note.commit()
    }

    #[test]
    fn merkle_proof_verification_succeeds() {
        let (pk, vk) = MerkleProofCircuit::generate_prepared_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness([b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());
        // We will incrementally add notes to the state commitment tree, checking the merkle proofs verify
        // at each step.
        let mut sct = tct::Tree::new();

        for _ in 0..5 {
            let note_commitment = make_random_note_commitment(address);
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let circuit = MerkleProofCircuit {
                state_commitment_proof,
                anchor,
            };
            let proof =
                Groth16::prove(&pk, circuit, &mut rng).expect("should be able to form proof");

            let proof_result = Groth16::verify_with_processed_vk(&vk, &[Fq::from(anchor)], &proof);
            assert!(proof_result.is_ok());
        }

        sct.end_block().expect("can end block");
        for _ in 0..100 {
            let note_commitment = make_random_note_commitment(address);
            sct.insert(tct::Witness::Forget, note_commitment).unwrap();
        }

        for _ in 0..5 {
            let note_commitment = make_random_note_commitment(address);
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let circuit = MerkleProofCircuit {
                state_commitment_proof,
                anchor,
            };
            let proof =
                Groth16::prove(&pk, circuit, &mut rng).expect("should be able to form proof");

            let proof_result = Groth16::verify_with_processed_vk(&vk, &[Fq::from(anchor)], &proof);
            assert!(proof_result.is_ok());
        }

        sct.end_epoch().expect("can end epoch");
        for _ in 0..100 {
            let note_commitment = make_random_note_commitment(address);
            sct.insert(tct::Witness::Forget, note_commitment).unwrap();
        }

        for _ in 0..5 {
            let note_commitment = make_random_note_commitment(address);
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let circuit = MerkleProofCircuit {
                state_commitment_proof,
                anchor,
            };
            let proof =
                Groth16::prove(&pk, circuit, &mut rng).expect("should be able to form proof");

            let proof_result = Groth16::verify_with_processed_vk(&vk, &[Fq::from(anchor)], &proof);
            assert!(proof_result.is_ok());
        }
    }
}
