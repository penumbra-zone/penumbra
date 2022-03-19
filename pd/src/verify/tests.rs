use ark_ff::Zero;
use penumbra_crypto::{
    asset,
    keys::{SeedPhrase, SpendKey, SpendSeed},
    memo::MemoPlaintext,
    Fq, Note, Value,
};
use penumbra_transaction::Transaction;
use rand_core::OsRng;

use super::*;

#[test]
fn test_transaction_succeeds_if_values_balance() {
    let mut rng = OsRng;
    let seed_phrase = SeedPhrase::generate(&mut rng);
    let spend_seed = SpendSeed::from_seed_phrase(seed_phrase, 0);
    let sk_sender = SpendKey::new(spend_seed);
    let fvk_sender = sk_sender.full_viewing_key();
    let ovk_sender = fvk_sender.outgoing();
    let (send_addr, _) = fvk_sender.incoming().payment_address(0u64.into());

    let seed_phrase = SeedPhrase::generate(&mut rng);
    let spend_seed = SpendSeed::from_seed_phrase(seed_phrase, 0);
    let sk_recipient = SpendKey::new(spend_seed);
    let fvk_recipient = sk_recipient.full_viewing_key();
    let ivk_recipient = fvk_recipient.incoming();
    let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

    let output_value = Value {
        amount: 10,
        asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
    };
    let spend_value = Value {
        amount: 20,
        asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
    };
    // The note was previously sent to the sender.
    let note = Note::from_parts(
        *send_addr.diversifier(),
        *send_addr.transmission_key(),
        spend_value,
        Fq::zero(),
    )
    .expect("transmission key is valid");
    let note_commitment = note.commit();

    let mut nct = penumbra_tct::Eternity::new();
    nct.insert(penumbra_tct::Keep, note_commitment);
    let anchor = nct.root();

    let transaction = Transaction::build_with_root(anchor)
        .set_fee(10)
        .set_chain_id("penumbra".to_string())
        .add_output(
            &mut rng,
            &dest,
            output_value,
            MemoPlaintext::default(),
            ovk_sender,
        )
        .add_spend(&mut rng, &nct, &sk_sender, note)
        .expect("note is in nct")
        .finalize(&mut rng)
        .expect("transaction created ok");

    let _pending_tx = transaction
        .verify_stateless()
        .expect("stateless verification should pass");
}
