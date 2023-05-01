use std::str::FromStr;

use super::*;
use ark_ff::PrimeField;
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use decaf377::{r1cs::FqVar, Bls12_377, Fq, Fr};
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_crypto::{
    asset,
    keys::{SeedPhrase, SpendKey},
    note, rdsa,
    stake::{IdentityKey, Penalty, UnbondingToken},
    Address, Amount, Balance, Fee, Note, Rseed, Value,
};
use penumbra_proto::core::crypto::v1alpha1 as pb;
use penumbra_tct as tct;
use proptest::prelude::*;
use rand_core::OsRng;
use tct::Commitment;

use crate::{swap::SwapPlaintext, TradingPair};

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
