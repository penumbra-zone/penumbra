//! Tests guard against drift in the current constraints versus the provided
//! proving/verification key.

use decaf377::{Fq, Fr};
use decaf377_rdsa::{SigningKey, SpendAuth, VerificationKey};
use penumbra_asset::{asset, Balance, Value};
use penumbra_dex::swap::proof::{SwapProofPrivate, SwapProofPublic};
use penumbra_dex::swap_claim::{SwapClaimProofPrivate, SwapClaimProofPublic};
use penumbra_dex::{
    swap::proof::SwapProof, swap::SwapPlaintext, swap_claim::proof::SwapClaimProof,
    BatchSwapOutputData, TradingPair,
};
use penumbra_fee::Fee;
use penumbra_governance::{
    DelegatorVoteProof, DelegatorVoteProofPrivate, DelegatorVoteProofPublic,
};
use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_num::Amount;
use penumbra_proof_params::{
    CONVERT_PROOF_PROVING_KEY, CONVERT_PROOF_VERIFICATION_KEY, DELEGATOR_VOTE_PROOF_PROVING_KEY,
    DELEGATOR_VOTE_PROOF_VERIFICATION_KEY, NULLIFIER_DERIVATION_PROOF_PROVING_KEY,
    NULLIFIER_DERIVATION_PROOF_VERIFICATION_KEY, OUTPUT_PROOF_PROVING_KEY,
    OUTPUT_PROOF_VERIFICATION_KEY, SPEND_PROOF_PROVING_KEY, SPEND_PROOF_VERIFICATION_KEY,
    SWAPCLAIM_PROOF_PROVING_KEY, SWAPCLAIM_PROOF_VERIFICATION_KEY, SWAP_PROOF_PROVING_KEY,
    SWAP_PROOF_VERIFICATION_KEY,
};
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::output::{OutputProofPrivate, OutputProofPublic};
use penumbra_shielded_pool::Note;
use penumbra_shielded_pool::{
    NullifierDerivationProof, NullifierDerivationProofPrivate, NullifierDerivationProofPublic,
    OutputProof, SpendProof, SpendProofPrivate, SpendProofPublic,
};
use penumbra_stake::undelegate_claim::{UndelegateClaimProofPrivate, UndelegateClaimProofPublic};
use penumbra_stake::{IdentityKey, Penalty, UnbondingToken, UndelegateClaimProof};
use penumbra_tct as tct;
use rand_core::OsRng;

#[test]
fn spend_proof_parameters_vs_current_spend_circuit() {
    let pk = &*SPEND_PROOF_PROVING_KEY;
    let vk = &*SPEND_PROOF_VERIFICATION_KEY;

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
    let fvk_sender = sk_sender.full_viewing_key();
    let ivk_sender = fvk_sender.incoming();
    let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

    let value_to_send = Value {
        amount: 1u64.into(),
        asset_id: asset::Cache::with_known_assets()
            .get_unit("upenumbra")
            .unwrap()
            .id(),
    };

    let note = Note::generate(&mut OsRng, &sender, value_to_send);
    let note_commitment = note.commit();
    let spend_auth_randomizer = Fr::rand(&mut OsRng);
    let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
    let nk = *sk_sender.nullifier_key();
    let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
    let mut sct = tct::Tree::new();
    sct.insert(tct::Witness::Keep, note_commitment).unwrap();
    let anchor = sct.root();
    let state_commitment_proof = sct.witness(note_commitment).unwrap();
    let v_blinding = Fr::rand(&mut OsRng);
    let balance_commitment = value_to_send.commit(v_blinding);
    let rk: VerificationKey<SpendAuth> = rsk.into();
    let nullifier = Nullifier::derive(&nk, 0.into(), &note_commitment);

    // Random elements to provide ZK (see Section 3.2 Groth16 paper, bottom of page 17)
    let blinding_r = Fq::rand(&mut OsRng);
    let blinding_s = Fq::rand(&mut OsRng);
    let public = SpendProofPublic {
        anchor,
        balance_commitment,
        nullifier,
        rk,
    };
    let private = SpendProofPrivate {
        state_commitment_proof,
        note,
        v_blinding,
        spend_auth_randomizer,
        ak,
        nk,
    };
    let proof = SpendProof::prove(blinding_r, blinding_s, pk, public.clone(), private)
        .expect("can create proof");

    let proof_result = proof.verify(vk, public);
    assert!(proof_result.is_ok());
}

#[test]
fn delegator_vote_proof_parameters_vs_current_delegator_vote_circuit() {
    let pk = &*DELEGATOR_VOTE_PROOF_PROVING_KEY;
    let vk = &*DELEGATOR_VOTE_PROOF_VERIFICATION_KEY;

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
    let fvk_sender = sk_sender.full_viewing_key();
    let ivk_sender = fvk_sender.incoming();
    let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

    let value_to_send = Value {
        amount: 2u64.into(),
        asset_id: asset::Cache::with_known_assets()
            .get_unit("upenumbra")
            .unwrap()
            .id(),
    };

    let note = Note::generate(&mut OsRng, &sender, value_to_send);
    let note_commitment = note.commit();
    let spend_auth_randomizer = Fr::rand(&mut OsRng);
    let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
    let nk = *sk_sender.nullifier_key();
    let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
    let mut sct = tct::Tree::new();

    sct.insert(tct::Witness::Keep, note_commitment).unwrap();
    let anchor = sct.root();
    let state_commitment_proof = sct.witness(note_commitment).unwrap();
    sct.end_epoch().unwrap();

    let first_note_commitment = Note::generate(&mut OsRng, &sender, value_to_send).commit();
    sct.insert(tct::Witness::Keep, first_note_commitment)
        .unwrap();
    let start_position = sct.witness(first_note_commitment).unwrap().position();

    let balance_commitment = value_to_send.commit(Fr::from(0u64));
    let rk: VerificationKey<SpendAuth> = rsk.into();
    let nf = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

    let blinding_r = Fq::rand(&mut OsRng);
    let blinding_s = Fq::rand(&mut OsRng);

    let public = DelegatorVoteProofPublic {
        anchor,
        balance_commitment,
        nullifier: nf,
        rk,
        start_position,
    };
    let private = DelegatorVoteProofPrivate {
        state_commitment_proof,
        note,
        v_blinding: Fr::from(0u64),
        spend_auth_randomizer,
        ak,
        nk,
    };
    let proof = DelegatorVoteProof::prove(blinding_r, blinding_s, pk, public.clone(), private)
        .expect("can create proof");

    let proof_result = proof.verify(vk, public);
    assert!(proof_result.is_ok());
}

#[test]
fn swap_proof_parameters_vs_current_swap_circuit() {
    let pk = &*SWAP_PROOF_PROVING_KEY;
    let vk = &*SWAP_PROOF_VERIFICATION_KEY;

    let mut rng = OsRng;

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
    let fvk_recipient = sk_recipient.full_viewing_key();
    let ivk_recipient = fvk_recipient.incoming();
    let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

    let trading_pair = TradingPair::new(gm.id(), gn.id());

    let delta_1 = Amount::from(100_000u64);
    let delta_2 = Amount::from(0u64);
    let fee = Fee::default();
    let fee_blinding = Fr::rand(&mut OsRng);

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
        asset_id: swap_plaintext.trading_pair.asset_2(),
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

    let public = SwapProofPublic {
        balance_commitment,
        swap_commitment,
        fee_commitment,
    };
    let private = SwapProofPrivate {
        fee_blinding,
        swap_plaintext,
    };

    let blinding_r = Fq::rand(&mut OsRng);
    let blinding_s = Fq::rand(&mut OsRng);
    let proof = SwapProof::prove(blinding_r, blinding_s, pk, public.clone(), private)
        .expect("can create proof");

    let proof_result = proof.verify(vk, public);

    assert!(proof_result.is_ok());
}

#[test]
fn swap_claim_parameters_vs_current_swap_claim_circuit() {
    let pk = &*SWAPCLAIM_PROOF_PROVING_KEY;
    let vk = &*SWAPCLAIM_PROOF_VERIFICATION_KEY;

    let mut rng = OsRng;

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
    let fvk_recipient = sk_recipient.full_viewing_key();
    let ivk_recipient = fvk_recipient.incoming();
    let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());
    let nk = *sk_recipient.nullifier_key();

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
    let trading_pair = TradingPair::new(gm.id(), gn.id());

    let delta_1_i = Amount::from(2u64);
    let delta_2_i = Amount::from(0u64);
    let fee = Fee::default();

    let swap_plaintext = SwapPlaintext::new(
        &mut rng,
        trading_pair,
        delta_1_i,
        delta_2_i,
        fee,
        claim_address,
    );
    let claim_fee = swap_plaintext.clone().claim_fee;
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
        delta_1: Amount::from(100u64),
        delta_2: Amount::from(100u64),
        lambda_1: Amount::from(50u64),
        lambda_2: Amount::from(25u64),
        unfilled_1: Amount::from(23u64),
        unfilled_2: Amount::from(50u64),
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

    let blinding_r = Fq::rand(&mut rng);
    let blinding_s = Fq::rand(&mut rng);

    let proof = SwapClaimProof::prove(blinding_r, blinding_s, pk, public.clone(), private)
        .expect("can create proof");

    let proof_result = proof.verify(vk, public);

    assert!(proof_result.is_ok());
}

#[test]
fn output_proof_parameters_vs_current_output_circuit() {
    let pk = &*OUTPUT_PROOF_PROVING_KEY;
    let vk = &*OUTPUT_PROOF_VERIFICATION_KEY;

    let (public, private) = {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(OsRng);
        let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u32.into());

        let value_to_send = Value {
            amount: 1u64.into(),
            asset_id: asset::Cache::with_known_assets()
                .get_unit("upenumbra")
                .unwrap()
                .id(),
        };
        let balance_blinding = Fr::rand(&mut OsRng);

        let note = Note::generate(&mut rng, &dest, value_to_send);
        let note_commitment = note.commit();
        let balance_commitment = (-Balance::from(value_to_send)).commit(balance_blinding);

        let public = OutputProofPublic {
            balance_commitment,
            note_commitment,
        };
        let private = OutputProofPrivate {
            note,
            balance_blinding,
        };

        (public, private)
    };

    let blinding_r = Fq::rand(&mut OsRng);
    let blinding_s = Fq::rand(&mut OsRng);
    let proof = OutputProof::prove(blinding_r, blinding_s, pk, public.clone(), private)
        .expect("can create proof");

    let proof_result = proof.verify(vk, public);

    assert!(proof_result.is_ok());
}

#[test]
fn nullifier_derivation_parameters_vs_current_nullifier_derivation_circuit() {
    let pk = &*NULLIFIER_DERIVATION_PROOF_PROVING_KEY;
    let vk = &*NULLIFIER_DERIVATION_PROOF_VERIFICATION_KEY;

    let mut rng = OsRng;

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
    let fvk_sender = sk_sender.full_viewing_key();
    let ivk_sender = fvk_sender.incoming();
    let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

    let value_to_send = Value {
        amount: 1u128.into(),
        asset_id: asset::Cache::with_known_assets()
            .get_unit("upenumbra")
            .unwrap()
            .id(),
    };

    let note = Note::generate(&mut rng, &sender, value_to_send);
    let note_commitment = note.commit();
    let nk = *sk_sender.nullifier_key();
    let mut sct = tct::Tree::new();

    sct.insert(tct::Witness::Keep, note_commitment).unwrap();
    let state_commitment_proof = sct.witness(note_commitment).unwrap();
    let position = state_commitment_proof.position();
    let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

    let public = NullifierDerivationProofPublic {
        position,
        note_commitment,
        nullifier,
    };
    let private = NullifierDerivationProofPrivate { nk };
    let proof = NullifierDerivationProof::prove(&mut rng, pk, public.clone(), private)
        .expect("can create proof");

    let proof_result = proof.verify(vk, public);

    assert!(proof_result.is_ok());
}

#[test]
fn undelegate_claim_parameters_vs_current_undelegate_claim_circuit() {
    let pk = &*CONVERT_PROOF_PROVING_KEY;
    let vk = &*CONVERT_PROOF_VERIFICATION_KEY;

    let mut rng = OsRng;

    let (public, private) = {
        let sk = SigningKey::new_from_field(Fr::from(1u8));
        let balance_blinding = Fr::from(1u8);
        let value1_amount = 1u64;
        let penalty_amount = 1u64;
        let validator_identity = IdentityKey((&sk).into());
        let unbonding_amount = Amount::from(value1_amount);

        let start_epoch_index = 1;
        let unbonding_token = UnbondingToken::new(validator_identity, start_epoch_index);
        let unbonding_id = unbonding_token.id();
        let penalty = Penalty::from_bps_squared(penalty_amount);
        let balance = penalty.balance_for_claim(unbonding_id, unbonding_amount);
        let balance_commitment = balance.commit(balance_blinding);

        (
            UndelegateClaimProofPublic {
                balance_commitment,
                unbonding_id,
                penalty,
            },
            UndelegateClaimProofPrivate {
                unbonding_amount,
                balance_blinding,
            },
        )
    };

    let blinding_r = Fq::rand(&mut rng);
    let blinding_s = Fq::rand(&mut rng);

    let proof = UndelegateClaimProof::prove(blinding_r, blinding_s, pk, public.clone(), private)
        .expect("can create proof");

    let proof_result = proof.verify(vk, public);

    assert!(proof_result.is_ok());
}
