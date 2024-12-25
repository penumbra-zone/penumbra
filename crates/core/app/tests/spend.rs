mod common;

use self::common::TempStorageExt;
use cnidarium::{ArcStateDeltaExt, StateDelta, TempStorage};
use cnidarium_component::{ActionHandler as _, Component};
use decaf377::{Fq, Fr};
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_sdk_asset::Value;
use penumbra_sdk_compact_block::component::CompactBlockManager;
use penumbra_sdk_keys::{keys::NullifierKey, test_keys};
use penumbra_sdk_mock_client::MockClient;
use penumbra_sdk_num::Amount;
use penumbra_sdk_sct::{
    component::{clock::EpochManager, source::SourceContext},
    epoch::Epoch,
};
use penumbra_sdk_shielded_pool::{
    component::ShieldedPool, Note, SpendPlan, SpendProof, SpendProofPrivate, SpendProofPublic,
};
use penumbra_sdk_txhash::{EffectHash, TransactionContext};
use rand_core::{OsRng, SeedableRng};
use std::{ops::Deref, sync::Arc};
use tendermint::abci;

#[tokio::test]
async fn spend_happy_path() -> anyhow::Result<()> {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new_with_penumbra_prefixes()
        .await?
        .apply_default_genesis()
        .await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let height = 1;

    // Precondition: This test uses the default genesis which has existing notes for the test keys.
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone());
    let sk = test_keys::SPEND_KEY.clone();
    client.sync_to(0, state.deref()).await?;
    let note = client.notes.values().next().unwrap().clone();
    let note_commitment = note.commit();
    let proof = client.sct.witness(note_commitment).unwrap();
    let root = client.sct.root();
    let tct_position = proof.position();

    // 1. Simulate BeginBlock
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_block_height(height);
    state_tx.put_epoch_by_height(
        height,
        Epoch {
            index: 0,
            start_height: 0,
        },
    );
    state_tx.apply();

    // 2. Create a Spend action
    let spend_plan = SpendPlan::new(&mut rng, note, tct_position);
    let dummy_effect_hash = [0u8; 64];
    let rsk = sk.spend_auth_key().randomize(&spend_plan.randomizer);
    let auth_sig = rsk.sign(&mut rng, dummy_effect_hash.as_ref());
    let spend = spend_plan.spend(&test_keys::FULL_VIEWING_KEY, auth_sig, proof, root);
    let transaction_context = TransactionContext {
        anchor: root,
        effect_hash: EffectHash(dummy_effect_hash),
    };

    // 3. Simulate execution of the Spend action
    spend.check_stateless(transaction_context).await?;
    spend.check_historical(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    spend.check_and_execute(&mut state_tx).await?;
    state_tx.apply();

    // 4. Execute EndBlock

    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };
    ShieldedPool::end_block(&mut state, &end_block).await;

    let mut state_tx = state.try_begin_transaction().unwrap();
    // ... and for the App, call `finish_block` to correctly write out the SCT with the data we'll use next.
    state_tx.finish_block().await.unwrap();

    state_tx.apply();

    Ok(())
}

// PoC for issue surfaced in zellic audit: https://github.com/penumbra-zone/penumbra/issues/3859
// test that 0-value spends with invalid proofs are not accepted
#[tokio::test]
// Arkworks uses debug assertions that trigger if a constraint system is not satisfied.
// This is a bit annoying because this gets in the way of testing bad dummy spends.
// Indeed, this test passes when run in release-mode but panics in debug mode.
// We need the config attribute below:
#[cfg_attr(
    debug_assertions,
    should_panic = "assertion failed: cs.is_satisfied().unwrap()"
)]
async fn invalid_dummy_spend() {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new_with_penumbra_prefixes()
        .await
        .unwrap()
        .apply_default_genesis()
        .await
        .unwrap();
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let height = 1;

    // Precondition: This test uses the default genesis which has existing notes for the test keys.
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone());
    let sk = test_keys::SPEND_KEY.clone();
    client.sync_to(0, state.deref()).await.unwrap();
    let note = client.notes.values().next().unwrap().clone();

    let note_commitment = note.commit();
    let proof = client.sct.witness(note_commitment).unwrap();
    let root = client.sct.root();
    let tct_position = proof.position();

    // 1. Simulate BeginBlock
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_block_height(height);
    state_tx.put_epoch_by_height(
        height,
        Epoch {
            index: 0,
            start_height: 0,
        },
    );
    state_tx.apply();

    // 2. Create a Spend action
    let spend_plan = SpendPlan::new(&mut rng, note.clone(), tct_position);
    let dummy_effect_hash = [0u8; 64];
    let rsk = sk.spend_auth_key().randomize(&spend_plan.randomizer);
    let auth_sig = rsk.sign(&mut rng, dummy_effect_hash.as_ref());
    let mut spend = spend_plan.spend(&test_keys::FULL_VIEWING_KEY, auth_sig, proof.clone(), root);

    let note_zero_value = Note::from_parts(
        note.address(),
        Value {
            amount: Amount::from(0u64),
            asset_id: note.asset_id(),
        },
        note.rseed(),
    )
    .unwrap();

    let public = SpendProofPublic {
        anchor: root,
        balance_commitment: spend_plan.balance().commit(spend_plan.value_blinding),
        nullifier: spend_plan.nullifier(&test_keys::FULL_VIEWING_KEY),
        rk: spend_plan.rk(&test_keys::FULL_VIEWING_KEY),
    };

    // construct a proof for this spend using only public information, attempting to prove a spend
    // of a dummy note.
    let ak = VerificationKey::<SpendAuth>::try_from([0u8; 32]).unwrap();
    let nk = NullifierKey(Fq::rand(&mut OsRng));

    let private = SpendProofPrivate {
        state_commitment_proof: proof,
        note: note_zero_value,
        v_blinding: Fr::rand(&mut OsRng),
        spend_auth_randomizer: Fr::rand(&mut OsRng),
        ak,
        nk,
    };
    let bad_proof = SpendProof::prove(
        Fq::rand(&mut OsRng),
        Fq::rand(&mut OsRng),
        &penumbra_sdk_proof_params::SPEND_PROOF_PROVING_KEY,
        public,
        private,
    )
    .expect("can generate ZKSpendProof");

    spend.proof = bad_proof;

    let transaction_context = TransactionContext {
        anchor: root,
        effect_hash: EffectHash(dummy_effect_hash),
    };

    // 3. Simulate execution of the Spend action
    assert!(spend
        .check_stateless(transaction_context)
        .await
        .unwrap_err()
        .to_string()
        .contains("spend proof did not verify"));
}

/*
#[tokio::test]
#[should_panic(expected = "was already spent")]
async fn spend_duplicate_nullifier_previous_transaction() {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new_with_penumbra_prefixes()
        .await
        .expect("can start new temp storage")
        .apply_default_genesis()
        .await
        .expect("can apply default genesis");
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let height = 1;

    // Precondition: This test uses the default genesis which has existing notes for the test keys.
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone());
    let sk = test_keys::SPEND_KEY.clone();
    client
        .sync_to(0, state.deref())
        .await
        .expect("can sync to genesis");
    let note = client.notes.values().next().unwrap().clone();
    let note_commitment = note.commit();
    let proof = client.sct.witness(note_commitment).unwrap();
    let root = client.sct.root();
    let tct_position = proof.position();

    // 1. Simulate BeginBlock
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_block_height(height);
    state_tx.put_epoch_by_height(
        height,
        Epoch {
            index: 0,
            start_height: 0,
        },
    );
    state_tx.apply();

    // 2. Create a Spend action - This is the first spend of this note.
    let spend_plan = SpendPlan::new(&mut rng, note.clone(), tct_position);
    let dummy_effect_hash = [0u8; 64];
    let rsk = sk.spend_auth_key().randomize(&spend_plan.randomizer);
    let auth_sig = rsk.sign(&mut rng, dummy_effect_hash.as_ref());
    let spend = spend_plan.spend(&test_keys::FULL_VIEWING_KEY, auth_sig, proof.clone(), root);
    let transaction_context = TransactionContext {
        anchor: root,
        effect_hash: EffectHash(dummy_effect_hash),
    };

    // 3. Simulate execution of the Spend action
    spend
        .check_stateless(transaction_context)
        .await
        .expect("can apply first spend");
    spend
        .check_historical(state.clone())
        .await
        .expect("can apply first spend");
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    spend
        .check_and_execute(&mut state_tx)
        .await
        .expect("should be able to apply first spend");
    state_tx.apply();

    // 4. Create a second Spend action of the same note - This is a double spend.
    let spend_plan = SpendPlan::new(&mut rng, note, tct_position);
    let dummy_effect_hash = [0u8; 64];
    let rsk = sk.spend_auth_key().randomize(&spend_plan.randomizer);
    let auth_sig = rsk.sign(&mut rng, dummy_effect_hash.as_ref());
    let spend = spend_plan.spend(&test_keys::FULL_VIEWING_KEY, auth_sig, proof, root);
    let transaction_context = TransactionContext {
        anchor: root,
        effect_hash: EffectHash(dummy_effect_hash),
    };

    // 5. Simulate execution of the double spend - the test should panic here
    spend
        .check_stateless(transaction_context)
        .await
        .expect("check stateless should succeed");
    spend.check_historical(state.clone()).await.unwrap();
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(2u8);
    spend.check_and_execute(&mut state_tx).await.unwrap();
    state_tx.apply();
}

#[tokio::test]
#[should_panic(expected = "Duplicate nullifier in transaction")]
async fn spend_duplicate_nullifier_same_transaction() {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new_with_penumbra_prefixes()
        .await
        .expect("can start new temp storage")
        .apply_default_genesis()
        .await
        .expect("can apply default genesis");
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let height = 1;

    // Precondition: This test uses the default genesis which has existing notes for the test keys.
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone());
    let sk = test_keys::SPEND_KEY.clone();
    client
        .sync_to(0, state.deref())
        .await
        .expect("can sync to genesis");
    let note = client.notes.values().next().unwrap().clone();
    let note_commitment = note.commit();
    let proof = client.sct.witness(note_commitment).unwrap();
    let root = client.sct.root();
    let tct_position = proof.position();

    // 1. Simulate BeginBlock
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_block_height(height);
    state_tx.put_epoch_by_height(
        height,
        Epoch {
            index: 0,
            start_height: 0,
        },
    );
    state_tx.apply();

    // 2. Create a Spend action - This is the first spend of this note.
    let spend_plan = SpendPlan::new(&mut rng, note.clone(), tct_position);
    let dummy_effect_hash = [0u8; 64];
    let rsk = sk.spend_auth_key().randomize(&spend_plan.randomizer);
    let auth_sig = rsk.sign(&mut rng, dummy_effect_hash.as_ref());
    let spend_1 = spend_plan.spend(&test_keys::FULL_VIEWING_KEY, auth_sig, proof.clone(), root);
    let mut synthetic_blinding_factor = spend_plan.value_blinding;

    // 3. Create a second Spend action of the same note - This is a double spend.
    let spend_plan = SpendPlan::new(&mut rng, note.clone(), tct_position);
    let dummy_effect_hash = [0u8; 64];
    let rsk = sk.spend_auth_key().randomize(&spend_plan.randomizer);
    let auth_sig = rsk.sign(&mut rng, dummy_effect_hash.as_ref());
    let spend_2 = spend_plan.spend(&test_keys::FULL_VIEWING_KEY, auth_sig, proof, root);
    synthetic_blinding_factor += spend_plan.value_blinding;

    // 4. We need to create an output to balance the transaction.
    let value = Value {
        amount: Amount::from(2u64) * note.amount(),
        asset_id: note.asset_id(),
    };
    let output_plan =
        penumbra_sdk_shielded_pool::OutputPlan::new(&mut rng, value, *test_keys::ADDRESS_1);
    let fvk = &test_keys::FULL_VIEWING_KEY;
    let memo_key = PayloadKey::random_key(&mut rng);
    let output = output_plan.output(fvk.outgoing(), &memo_key);
    synthetic_blinding_factor += output_plan.value_blinding;

    // 5. Construct a transaction with both spends that use the same note/nullifier.
    let transaction_body = TransactionBody {
        actions: vec![
            penumbra_sdk_transaction::Action::Spend(spend_1),
            penumbra_sdk_transaction::Action::Spend(spend_2),
            penumbra_sdk_transaction::Action::Output(output),
        ],
        transaction_parameters: TransactionParameters::default(),
        detection_data: None,
        memo: None,
    };
    let binding_signing_key = SigningKey::from(synthetic_blinding_factor);
    let auth_hash = transaction_body.auth_hash();
    let binding_sig = binding_signing_key.sign(rng, auth_hash.as_bytes());
    let transaction = Transaction {
        transaction_body,
        binding_sig,
        anchor: root,
    };

    // 6. Simulate execution of the transaction - the test should panic here
    transaction.check_stateless(()).await.unwrap();
}
 */
