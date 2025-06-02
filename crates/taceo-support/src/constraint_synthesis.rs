use anyhow::Result;
use ark_relations::r1cs::{self, ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef};

use decaf377::{Fq, Fr};
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_sdk_asset::{asset, Value};
use penumbra_sdk_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_sdk_num::Amount;
use penumbra_sdk_sct::Nullifier;
use penumbra_sdk_shielded_pool::{Note, Rseed, SpendCircuit, SpendProofPrivate, SpendProofPublic};
use penumbra_sdk_tct as tct;
use rand::{Rng, RngCore};
use rand_core::OsRng;

pub fn generate_valid_spend_inputs() -> (SpendProofPublic, SpendProofPrivate) {
    // Generate random field elements
    let mut rng = OsRng;
    let v_blinding = Fr::rand(&mut rng);
    let spend_auth_randomizer = Fr::rand(&mut rng);

    // Random asset ID and address index
    let asset_id64 = rng.next_u64();
    let address_index = rng.next_u32();
    let amount = rng.next_u64();

    // Random seed phrase and rseed entropy
    let mut seed_phrase_randomness = [0u8; 32];
    rng.fill_bytes(&mut seed_phrase_randomness);
    let mut rseed_randomness = [0u8; 32];
    rng.fill_bytes(&mut rseed_randomness);

    // Random number of unrelated commitments
    let num_commitments = rng.gen_range(0..100);

    let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
    let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
    let fvk_sender = sk_sender.full_viewing_key();
    let ivk_sender = fvk_sender.incoming();
    let (sender, _dtk_d) = ivk_sender.payment_address(address_index.into());

    let value_to_send = Value {
        amount: Amount::from(amount),
        asset_id: asset::Id(Fq::from(asset_id64)),
    };
    let note = Note::from_parts(sender.clone(), value_to_send, Rseed(rseed_randomness))
        .expect("should be able to create note");
    let note_commitment = note.commit();

    let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
    let nk = *sk_sender.nullifier_key();
    let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();

    let mut sct = tct::Tree::new();

    for i in 0..num_commitments {
        let dummy_rseed = Rseed([i as u8; 32]);
        let dummy_commitment = Note::from_parts(sender.clone(), value_to_send, dummy_rseed)
            .expect("can create dummy note")
            .commit();
        sct.insert(tct::Witness::Keep, dummy_commitment)
            .expect("insert dummy note into SCT");
    }

    sct.insert(tct::Witness::Keep, note_commitment)
        .expect("insert note into SCT");

    let anchor = sct.root();
    let state_commitment_proof = sct
        .witness(note_commitment)
        .expect("can witness note commitment");

    let balance_commitment = value_to_send.commit(v_blinding);
    let rk: VerificationKey<SpendAuth> = rsk.into();
    let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

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

    (public, private)
}

pub fn generate_circuit_constraints(
    public: SpendProofPublic,
    private: SpendProofPrivate,
) -> Result<()> {
    let cs: ConstraintSystemRef<_> = ConstraintSystem::new_ref();
    let circuit = SpendCircuit { public, private };
    cs.set_optimization_goal(r1cs::OptimizationGoal::Constraints);
    circuit
        .clone()
        .generate_constraints(cs.clone())
        .expect("can generate constraints from circuit");
    cs.finalize();
    if !cs.is_satisfied()? {
        anyhow::bail!("constraints are not satisfied");
    }

    println!("circuit: {:?}", circuit);

    Ok(())
}
