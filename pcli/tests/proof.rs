//! Tests guard against drift in the current constraints versus the provided
//! proving/verification key.

use ark_ff::UniformRand;
use decaf377::Fr;
use penumbra_crypto::{
    asset,
    dex::{swap::SwapPlaintext, TradingPair},
    keys::{SeedPhrase, SpendKey},
    proofs::groth16::{OutputProof, SpendProof, SwapProof},
    rdsa::{SpendAuth, VerificationKey},
    transaction::Fee,
    Amount, Balance, Note, Value,
};
use penumbra_proof_params::{
    OUTPUT_PROOF_PROVING_KEY, OUTPUT_PROOF_VERIFICATION_KEY, SPEND_PROOF_PROVING_KEY,
    SPEND_PROOF_VERIFICATION_KEY, SWAP_PROOF_PROVING_KEY, SWAP_PROOF_VERIFICATION_KEY,
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
        &pk,
        note,
        v_blinding,
        balance_commitment,
        note_commitment,
    )
    .expect("can create proof");

    let proof_result = proof.verify(&vk, balance_commitment, note_commitment);

    assert!(proof_result.is_ok());
}
