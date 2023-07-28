use std::{ops::Deref, sync::Arc};

use crate::{app::App, MockClient, TempStorageExt};
use penumbra_asset::{Value, STAKING_TOKEN_ASSET_ID};
use penumbra_chain::{
    component::{StateReadExt, StateWriteExt},
    test_keys, EffectHash, TransactionContext,
};
use penumbra_component::{ActionHandler, Component};
use penumbra_fee::Fee;
use penumbra_keys::Address;
use penumbra_num::Amount;
use penumbra_sct::component::{SctManager, StateReadExt as _};
use penumbra_shielded_pool::{component::ShieldedPool, Note, SpendPlan};
use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};
use penumbra_transaction::Transaction;
use rand_core::SeedableRng;
use tendermint::abci;

#[tokio::test]
async fn spend_happy_path() -> anyhow::Result<()> {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let height = 1;

    // Precondition: This test uses the default genesis which has existing notes for the test keys.
    let mut client = MockClient::new(test_keys::FULL_VIEWING_KEY.clone());
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
        penumbra_chain::Epoch {
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
    spend.check_stateful(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    spend.execute(&mut state_tx).await?;
    state_tx.apply();

    // 4. Execute EndBlock

    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };
    ShieldedPool::end_block(&mut state, &end_block).await;

    let mut state_tx = state.try_begin_transaction().unwrap();
    // ... and for the App, call `finish_block` to correctly write out the SCT with the data we'll use next.
    App::finish_block(&mut state_tx).await;

    state_tx.apply();

    Ok(())
}
