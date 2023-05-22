//! Tests guard against drift in the current constraints versus the provided
//! proving/verification key.

use ark_ff::UniformRand;
use decaf377::Fr;
use penumbra_crypto::{
    asset,
    keys::{SeedPhrase, SpendKey},
    proofs::groth16::{
        DelegatorVoteProof, NullifierDerivationProof, OutputProof, SpendProof, UndelegateClaimProof,
    },
    rdsa::{self, SpendAuth, VerificationKey},
    stake::{IdentityKey, Penalty, UnbondingToken},
    Amount, Balance, Fee, Note, Value,
};
use penumbra_dex::{swap::proof::SwapProof, swap::SwapPlaintext, TradingPair};
use penumbra_proof_params::{
    DELEGATOR_VOTE_PROOF_PROVING_KEY, DELEGATOR_VOTE_PROOF_VERIFICATION_KEY,
    NULLIFIER_DERIVATION_PROOF_PROVING_KEY, NULLIFIER_DERIVATION_PROOF_VERIFICATION_KEY,
    OUTPUT_PROOF_PROVING_KEY, OUTPUT_PROOF_VERIFICATION_KEY, SPEND_PROOF_PROVING_KEY,
    SPEND_PROOF_VERIFICATION_KEY, SWAP_PROOF_PROVING_KEY, SWAP_PROOF_VERIFICATION_KEY,
    UNDELEGATECLAIM_PROOF_PROVING_KEY, UNDELEGATECLAIM_PROOF_VERIFICATION_KEY,
};
use penumbra_tct as tct;
use rand_core::OsRng;

#[test]
fn spend_proof_parameters_vs_current_spend_circuit() {
    let pk = &*SPEND_PROOF_PROVING_KEY;
    let vk = &*SPEND_PROOF_VERIFICATION_KEY;

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
    let fvk_sender = sk_sender.full_viewing_key();
    let ivk_sender = fvk_sender.incoming();
    let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

    let value_to_send = Value {
        amount: 1u64.into(),
        asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
    };

    let note = Note::generate(&mut OsRng, &sender, value_to_send);
    let note_commitment = note.commit();
    let spend_auth_randomizer = Fr::rand(&mut OsRng);
    let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
    let nk = *sk_sender.nullifier_key();
    let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
    let mut nct = tct::Tree::new();
    nct.insert(tct::Witness::Keep, note_commitment).unwrap();
    let anchor = nct.root();
    let note_commitment_proof = nct.witness(note_commitment).unwrap();
    let v_blinding = Fr::rand(&mut OsRng);
    let balance_commitment = value_to_send.commit(v_blinding);
    let rk: VerificationKey<SpendAuth> = rsk.into();
    let nf = nk.derive_nullifier(0.into(), &note_commitment);

    let proof = SpendProof::prove(
        &mut OsRng,
        pk,
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

    let proof_result = proof.verify(vk, anchor, balance_commitment, nf, rk);
    assert!(proof_result.is_ok());
}

#[test]
fn delegator_vote_proof_parameters_vs_current_delegator_vote_circuit() {
    let pk = &*DELEGATOR_VOTE_PROOF_PROVING_KEY;
    let vk = &*DELEGATOR_VOTE_PROOF_VERIFICATION_KEY;

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
    let fvk_sender = sk_sender.full_viewing_key();
    let ivk_sender = fvk_sender.incoming();
    let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

    let value_to_send = Value {
        amount: 2u64.into(),
        asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
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
    let nf = nk.derive_nullifier(state_commitment_proof.position(), &note_commitment);

    let proof = DelegatorVoteProof::prove(
        &mut OsRng,
        pk,
        state_commitment_proof,
        note,
        spend_auth_randomizer,
        ak,
        nk,
        anchor,
        balance_commitment,
        nf,
        rk,
        start_position,
    )
    .expect("can create proof");

    let proof_result = proof.verify(vk, anchor, balance_commitment, nf, rk, start_position);
    assert!(proof_result.is_ok());
}

#[test]
fn swap_proof_parameters_vs_current_swap_circuit() {
    let pk = &*SWAP_PROOF_PROVING_KEY;
    let vk = &*SWAP_PROOF_VERIFICATION_KEY;

    let mut rng = OsRng;

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
    let fvk_recipient = sk_recipient.full_viewing_key();
    let ivk_recipient = fvk_recipient.incoming();
    let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
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

    let proof = SwapProof::prove(
        &mut rng,
        pk,
        swap_plaintext,
        fee_blinding,
        balance_commitment,
        swap_commitment,
        fee_commitment,
    )
    .expect("can create proof");

    let proof_result = proof.verify(vk, balance_commitment, swap_commitment, fee_commitment);

    assert!(proof_result.is_ok());
}

#[test]
fn output_proof_parameters_vs_current_output_circuit() {
    let pk = &*OUTPUT_PROOF_PROVING_KEY;
    let vk = &*OUTPUT_PROOF_VERIFICATION_KEY;

    let mut rng = OsRng;

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
    let fvk_recipient = sk_recipient.full_viewing_key();
    let ivk_recipient = fvk_recipient.incoming();
    let (dest, _dtk_d) = ivk_recipient.payment_address(0u32.into());

    let value_to_send = Value {
        amount: 1u64.into(),
        asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
    };
    let v_blinding = Fr::rand(&mut OsRng);

    let note = Note::generate(&mut rng, &dest, value_to_send);
    let note_commitment = note.commit();
    let balance_commitment = (-Balance::from(value_to_send)).commit(v_blinding);

    let proof = OutputProof::prove(
        &mut rng,
        pk,
        note,
        v_blinding,
        balance_commitment,
        note_commitment,
    )
    .expect("can create proof");

    let proof_result = proof.verify(vk, balance_commitment, note_commitment);

    assert!(proof_result.is_ok());
}

#[test]
fn nullifier_derivation_parameters_vs_current_nullifier_derivation_circuit() {
    let pk = &*NULLIFIER_DERIVATION_PROOF_PROVING_KEY;
    let vk = &*NULLIFIER_DERIVATION_PROOF_VERIFICATION_KEY;

    let mut rng = OsRng;

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
    let fvk_sender = sk_sender.full_viewing_key();
    let ivk_sender = fvk_sender.incoming();
    let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

    let value_to_send = Value {
        amount: 1u128.into(),
        asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
    };

    let note = Note::generate(&mut rng, &sender, value_to_send);
    let note_commitment = note.commit();
    let nk = *sk_sender.nullifier_key();
    let mut sct = tct::Tree::new();

    sct.insert(tct::Witness::Keep, note_commitment).unwrap();
    let state_commitment_proof = sct.witness(note_commitment).unwrap();
    let position = state_commitment_proof.position();
    let nullifier = nk.derive_nullifier(state_commitment_proof.position(), &note_commitment);

    let proof =
        NullifierDerivationProof::prove(&mut rng, pk, position, note.clone(), nk, nullifier)
            .expect("can create proof");

    let proof_result = proof.verify(vk, position, note, nullifier);

    assert!(proof_result.is_ok());
}

#[test]
fn undelegate_claim_parameters_vs_current_undelegate_claim_circuit() {
    let pk = &*UNDELEGATECLAIM_PROOF_PROVING_KEY;
    let vk = &*UNDELEGATECLAIM_PROOF_VERIFICATION_KEY;

    let mut rng = OsRng;

    let sk = rdsa::SigningKey::new_from_field(Fr::from(1u8));
    let balance_blinding = Fr::from(1u8);
    let value1_amount = 1u64;
    let penalty_amount = 1u64;
    let validator_identity = IdentityKey((&sk).into());
    let unbonding_amount = Amount::from(value1_amount);

    let start_epoch_index = 1;
    let unbonding_token = UnbondingToken::new(validator_identity, start_epoch_index);
    let unbonding_id = unbonding_token.id();
    let penalty = Penalty(penalty_amount);
    let balance = penalty.balance_for_claim(unbonding_id, unbonding_amount);
    let balance_commitment = balance.commit(balance_blinding);

    let proof = UndelegateClaimProof::prove(
        &mut rng,
        pk,
        unbonding_amount,
        balance_blinding,
        balance_commitment,
        unbonding_id,
        penalty,
    )
    .expect("can create proof");

    let proof_result = proof.verify(vk, balance_commitment, unbonding_id, penalty);

    assert!(proof_result.is_ok());
}
