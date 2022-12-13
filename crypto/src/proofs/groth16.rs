pub(crate) mod gadgets;
mod output;
mod spend;
mod traits;

pub use output::{OutputCircuit, OutputProof};
pub use spend::{SpendCircuit, SpendProof};
pub use traits::ParameterSetup;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        asset,
        keys::{SeedPhrase, SpendKey},
    };
    use decaf377::{Fq, Fr};
    use decaf377_ka as ka;
    use proptest::prelude::*;

    use decaf377_rdsa::{SpendAuth, VerificationKey};
    use penumbra_tct as tct;
    use rand_core::OsRng;

    use crate::{note, Note, Value};

    use ark_ff::{PrimeField, UniformRand};

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
    fn output_proof_happy_path(v_blinding in fr_strategy(), value_amount in 2..200u64, esk_inner in fr_strategy()) {
            let (pk, vk) = OutputCircuit::generate_test_parameters();

            let mut rng = OsRng;

            let seed_phrase = SeedPhrase::generate(rng);
            let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
            let fvk_recipient = sk_recipient.full_viewing_key();
            let ivk_recipient = fvk_recipient.incoming();
            let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

            let value_to_send = Value {
                amount: value_amount.into(),
                asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &dest, value_to_send);
            let note_commitment = note.commit();
            let esk = ka::Secret::new_from_field(esk_inner);
            let epk = esk.diversified_public(&note.diversified_generator());
            let balance_commitment = value_to_send.commit(v_blinding);

            let proof = OutputProof::prove(
                &mut rng,
                &pk,
                note,
                v_blinding,
                esk,
                balance_commitment,
                note_commitment,
                epk,
            )
            .expect("can create proof");

            let proof_result = proof.verify(&vk, balance_commitment, note_commitment, epk);

            assert!(proof_result.is_ok());
        }
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn output_proof_verification_note_commitment_integrity_failure(v_blinding in fr_strategy(), value_amount in 2..200u64, esk_inner in fr_strategy(), note_blinding in fq_strategy()) {
        let (pk, vk) = OutputCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &dest, value_to_send);
        let note_commitment = note.commit();
        let esk = ka::Secret::new_from_field(esk_inner);
        let epk = esk.diversified_public(&note.diversified_generator());
        let balance_commitment = value_to_send.commit(v_blinding);

        let proof = OutputProof::prove(
            &mut rng,
            &pk,
            note.clone(),
            v_blinding,
            esk,
            balance_commitment,
            note_commitment,
            epk,
        )
        .expect("can create proof");

        let incorrect_note_commitment = note::commitment(
            note_blinding,
            value_to_send,
            note.diversified_generator(),
            note.transmission_key_s(),
            note.clue_key(),
        );

        let proof_result = proof.verify(&vk, balance_commitment, incorrect_note_commitment, epk);

        assert!(proof_result.is_err());
    }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2))]
        #[test]
    fn output_proof_verification_balance_commitment_integrity_failure(v_blinding in fr_strategy(), value_amount in 2..200u64, esk_inner in fr_strategy(), incorrect_v_blinding in fr_strategy()) {
        let (pk, vk) = OutputCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &dest, value_to_send);
        let note_commitment = note.commit();
        let esk = ka::Secret::new_from_field(esk_inner);
        let epk = esk.diversified_public(&note.diversified_generator());
        let balance_commitment = value_to_send.commit(v_blinding);

        let proof = OutputProof::prove(
            &mut rng,
            &pk,
            note,
            v_blinding,
            esk,
            balance_commitment,
            note_commitment,
            epk,
        )
        .expect("can create proof");

        let incorrect_balance_commitment = value_to_send.commit(incorrect_v_blinding);

        let proof_result = proof.verify(&vk, incorrect_balance_commitment, note_commitment, epk);

        assert!(proof_result.is_err());
    }
        }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn output_proof_verification_ephemeral_public_key_integrity_failure(v_blinding in fr_strategy(), value_amount in 2..200u64, esk_inner in fr_strategy(), incorrect_esk_inner in fr_strategy()) {
        let (pk, vk) = OutputCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &dest, value_to_send);
        let note_commitment = note.commit();
        let esk = ka::Secret::new_from_field(esk_inner);
        let epk = esk.diversified_public(&note.diversified_generator());
        let balance_commitment = value_to_send.commit(v_blinding);

        let proof = OutputProof::prove(
            &mut rng,
            &pk,
            note.clone(),
            v_blinding,
            esk,
            balance_commitment,
            note_commitment,
            epk,
        )
        .expect("can create proof");

        let incorrect_esk = ka::Secret::new_from_field(incorrect_esk_inner);
        let incorrect_epk = incorrect_esk.diversified_public(&note.diversified_generator());

        let proof_result = proof.verify(&vk, balance_commitment, note_commitment, incorrect_epk);

        assert!(proof_result.is_err());
    }
    }

    #[test]
    /// Check that the `SpendProof` verification succeeds.
    fn spend_proof_verification_success() {
        let (pk, vk) = SpendCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u64.into());
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
        let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
        let mut nct = tct::Tree::new();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = nct.root();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);

        let proof = SpendProof::prove(
            &mut rng,
            &pk,
            note_commitment_proof,
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

    #[test]
    /// Check that the `SpendProof` verification fails when using an incorrect
    /// NCT root (`anchor`).
    fn spend_proof_verification_merkle_path_integrity_failure() {
        let (pk, vk) = SpendCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u64.into());
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
        let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
        let mut nct = tct::Tree::new();
        let incorrect_anchor = nct.root();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = nct.root();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);

        let proof = SpendProof::prove(
            &mut rng,
            &pk,
            note_commitment_proof,
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

    #[test]
    #[should_panic]
    /// Check that the `SpendProof` proof creation fails when the diversified address is wrong.
    fn spend_proof_verification_diversified_address_integrity_failure() {
        let (pk, _vk) = SpendCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);

        let v_blinding = Fr::rand(&mut rng);

        let wrong_seed_phrase = SeedPhrase::generate(rng);
        let wrong_sk_sender = SpendKey::from_seed_phrase(wrong_seed_phrase, 0);
        let wrong_fvk_sender = wrong_sk_sender.full_viewing_key();
        let wrong_ivk_sender = wrong_fvk_sender.incoming();
        let (wrong_sender, _dtk_d) = wrong_ivk_sender.payment_address(1u64.into());

        let value_to_send = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &wrong_sender, value_to_send);

        let note_commitment = note.commit();
        let spend_auth_randomizer = Fr::rand(&mut rng);
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let mut nct = tct::Tree::new();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = nct.root();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);

        // Circuit should be unsatisifiable if the diversified address integrity fails.
        // This will cause a panic.
        SpendProof::prove(
            &mut rng,
            &pk,
            note_commitment_proof,
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
        .expect("boom");
    }

    #[test]
    /// Check that the `SpendProof` verification fails, when using an
    /// incorrect nullifier.
    fn spend_proof_verification_nullifier_integrity_failure() {
        let (pk, vk) = SpendCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u64.into());
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
        let mut nct = tct::Tree::new();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = nct.root();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);

        let incorrect_nf = nk.derive_nullifier(5.into(), &note_commitment);

        let proof = SpendProof::prove(
            &mut rng,
            &pk,
            note_commitment_proof,
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

    #[test]
    /// Check that the `SpendProof` verification fails when using balance
    /// commitments with different blinding factors.
    fn spend_proof_verification_balance_commitment_integrity_failure() {
        let (pk, vk) = SpendCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u64.into());
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
        let mut nct = tct::Tree::new();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = nct.root();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);

        let proof = SpendProof::prove(
            &mut rng,
            &pk,
            note_commitment_proof,
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

        let incorrect_balance_commitment = value_to_send.commit(Fr::rand(&mut rng));

        let proof_result = proof.verify(&vk, anchor, incorrect_balance_commitment, nf, rk);
        assert!(proof_result.is_err());
    }

    #[test]
    /// Check that the `SpendProof` verification fails when the incorrect randomizable verification key is used.
    fn spend_proof_verification_fails_rk_integrity() {
        let (pk, vk) = SpendCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u64.into());
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
        let mut nct = tct::Tree::new();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = nct.root();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);

        let incorrect_spend_auth_randomizer = Fr::rand(&mut rng);
        let incorrect_rsk = sk_sender
            .spend_auth_key()
            .randomize(&incorrect_spend_auth_randomizer);
        let incorrect_rk: VerificationKey<SpendAuth> = incorrect_rsk.into();

        let proof = SpendProof::prove(
            &mut rng,
            &pk,
            note_commitment_proof,
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

    #[test]
    /// Check that the `SpendProof` verification always suceeds for dummy (zero value) spends.
    fn spend_proof_dummy_verification_suceeds() {
        let (pk, vk) = SpendCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u64.into());
        let v_blinding = Fr::rand(&mut rng);

        let value_to_send = Value {
            amount: 0u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

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
        // Using a random blinding factor here, but the proof will verify
        // since for dummies we only check if the value is zero, and choose
        // not to enforce the other equality constraint.
        let balance_commitment = value_to_send.commit(Fr::rand(&mut rng));
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);

        let proof = SpendProof::prove(
            &mut rng,
            &pk,
            note_commitment_proof,
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
