use std::{ops::Deref, sync::Arc};

use penumbra_chain::{
    component::{StateReadExt as _, StateWriteExt as _},
    test_keys,
};
use penumbra_component::{ActionHandler as _, Component};
use penumbra_crypto::{stake::IdentityKey, Amount, Fee, GovernanceKey};
use penumbra_dex::component::StateReadExt as _;
use penumbra_proto::{core::stake::v1alpha1::Validator as ProtoValidator, DomainType, Message};
use penumbra_shielded_pool::component::ShieldedPool;
use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};

use anyhow::Ok;
use async_trait::async_trait;

use penumbra_stake::{
    component::{Staking, StateReadExt as _, StateWriteExt as _},
    rate::RateData,
    validator::{self, Validator},
    Delegate, FundingStreams, Undelegate,
};
use rand_core::OsRng;
use tendermint::abci;

use crate::{app::App, MockClient, TempStorageExt};

/// Generate a hardcoded test ED25519 keypair for use with Tendermint.
fn test_tendermint_keypair() -> anyhow::Result<tendermint::PrivateKey> {
    let signing_key: [u8; 32] = [0; 32];
    let slice_signing_key = signing_key.as_slice();
    let priv_consensus_key = tendermint::PrivateKey::Ed25519(slice_signing_key.try_into()?);
    Ok(priv_consensus_key)
}

#[tokio::test]
/// Test that penalties are properly applied during unbonding periods.
async fn unbonding_slashing_penalties() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let mut height = 1;

    // 0. Simulate BeginBlock
    // TODO: Would be nice to call `App::begin_block` but creating the `abci::request::BeginBlock` is tricky
    // App::begin_block(&mut state_tx).await;
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

    // 1. Create a validator

    // Sign the validator definition with the wallet's spend key.
    let fvk = &test_keys::FULL_VIEWING_KEY;
    let sk = &test_keys::SPEND_KEY;
    let identity_key = IdentityKey(fvk.spend_verification_key().clone());
    let governance_key = GovernanceKey(identity_key.0);
    let consensus_key: tendermint::PublicKey = test_tendermint_keypair()?.public_key();
    let new_validator = Validator {
        identity_key,
        governance_key,
        consensus_key,
        name: "test validator".into(),
        website: "https://penumbra.zone".into(),
        description: "penumbra integration test".into(),
        enabled: true,
        funding_streams: FundingStreams::new(),
        sequence_number: 1,
    };
    let protobuf_serialized: ProtoValidator = new_validator.clone().into();
    let v_bytes = protobuf_serialized.encode_to_vec();
    let auth_sig = sk.spend_auth_key().sign(OsRng, &v_bytes);
    let vd = validator::Definition {
        validator: new_validator,
        auth_sig,
    };

    // Simulate constructing a new transaction and including the validator definition.
    vd.check_stateless(()).await?;
    vd.check_stateful(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    vd.execute(&mut state_tx).await?;
    state_tx.apply();

    // End the block prior to delegating...
    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };
    tracing::info!("END_BLOCK 1");
    // Execute EndBlock for the staking component.
    Staking::end_block(&mut state, &end_block).await;
    // To store the new validator, the epoch needs to end:
    Staking::end_epoch(&mut state).await?;

    let mut state_tx = state.try_begin_transaction().unwrap();
    // Call finish_block...
    App::finish_block(&mut state_tx).await;

    state_tx.apply();

    let mut client = MockClient::new(test_keys::FULL_VIEWING_KEY.clone());
    // TODO: generalize StateRead/StateWrite impls from impl for &S to impl for Deref<Target=S>
    client.sync_to(height, state.deref()).await?;
    tracing::info!("synced first time");

    // Increment the block height
    height = height + 1;

    // Simulate BeginBlock
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

    // 2. Delegate to the validator
    let unbonded_amount = 100u32;
    // Default rate data for new validator. Could pull from state instead.
    let cur_rate_data = RateData {
        identity_key: identity_key.clone(),
        epoch_index: 1,
        validator_reward_rate: 0,
        validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
    };
    let delegation_amount = cur_rate_data.delegation_amount(unbonded_amount.into());
    let delegate = Delegate {
        delegation_amount: delegation_amount.into(),
        epoch_index: 1,
        unbonded_amount: unbonded_amount.into(),
        validator_identity: identity_key.clone(),
    };

    // Simulate constructing a new transaction and including the validator delegation.
    delegate.check_stateless(()).await?;
    delegate.check_stateful(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    delegate.execute(&mut state_tx).await?;
    state_tx.apply();

    // End the block prior to undelegating...
    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };
    tracing::info!("END_BLOCK 2");
    // Execute EndBlock for the staking component, to actually execute the delegation...
    Staking::end_block(&mut state, &end_block).await;
    ShieldedPool::end_block(&mut state, &end_block).await;

    let mut state_tx = state.try_begin_transaction().unwrap();
    // Call finish_block...
    App::finish_block(&mut state_tx).await;

    state_tx.apply();

    // Increment the block height
    height = height + 1;

    // Simulate BeginBlock
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

    // Null case: no slashing. Undelegate immediately and see that there are no penalties.
    let undelegate = Undelegate {
        validator_identity: identity_key.clone(),
        start_epoch_index: 1,
        // TODO: should these be generated from the state?
        unbonded_amount: unbonded_amount.into(),
        delegation_amount: delegation_amount.into(),
    };

    // Simulate constructing a new transaction and including the validator undelegation.
    undelegate.check_stateless(()).await?;
    undelegate.check_stateful(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    undelegate.execute(&mut state_tx).await?;
    state_tx.apply();

    // End the block prior to undelegating...
    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };
    // Execute EndBlock for the staking component, to actually execute the undelegation...
    Staking::end_block(&mut state, &end_block).await;
    ShieldedPool::end_block(&mut state, &end_block).await;

    let mut state_tx = state.try_begin_transaction().unwrap();
    // Call finish_block...
    App::finish_block(&mut state_tx).await;

    state_tx.apply();

    // 6. Create an UndelegateClaim action

    // To do this, we need to have an auth path for the undelegate nft note, which
    // means we have to synchronize a client's view of the test chain's SCT
    // state.

    let epoch_duration = state.get_epoch_duration().await?;
    let mut client = MockClient::new(test_keys::FULL_VIEWING_KEY.clone());
    // TODO: generalize StateRead/StateWrite impls from impl for &S to impl for Deref<Target=S>
    client.sync_to(height, state.deref()).await?;

    // Increment the block height
    height = height + 1;

    // Simulate BeginBlock
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

    tracing::info!("notes: {:?}", client.notes());
    let commitment = undelegate.commitment();

    // let output_data = state.output_data(height, trading_pair).await?.unwrap();

    // let commitment = undelegate.body.payload.commitment;
    // let swap_auth_path = client.witness(commitment).unwrap();
    // let detected_plaintext = client.swap_by_commitment(&commitment).unwrap();
    // assert_eq!(plaintext, detected_plaintext);

    // let claim_plan = SwapClaimPlan {
    //     swap_plaintext: plaintext,
    //     position: swap_auth_path.position(),
    //     output_data,
    //     epoch_duration,
    // };
    // let claim = claim_plan.swap_claim(&test_keys::FULL_VIEWING_KEY, &swap_auth_path);

    // 3. Slash the validator

    // 4. Undelegate from the validator

    // 5. Undelegate claim

    // 6. Check that the undelegate claim penalties were applied correctly

    // if the slashing occurred before the undelegation, then there should be a 1:1 ratio between unbonding tokens and penumbra tokens after undelegation claim
    // if they occurred simultaneously, the penumbra tokens should have a slashing penalty applied to them relative to the amount of unbonding tokens
    // if the slashing occurred after the undelegation but before the end of the unbonding period, then there should also be a penalty
    // if the slashing occurred after the undelegation and also after or equal to the end of the unbonding period, then there should not be a penalty

    Ok(())
}
