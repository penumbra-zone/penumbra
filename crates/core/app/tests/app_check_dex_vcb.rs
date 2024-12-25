mod common;

use self::common::TempStorageExt;
use cnidarium::{ArcStateDeltaExt, StateDelta, TempStorage};
use cnidarium_component::ActionHandler;
use penumbra_sdk_asset::asset;
use penumbra_sdk_dex::{
    component::ValueCircuitBreakerRead,
    swap::{SwapPlaintext, SwapPlan},
    TradingPair,
};
use penumbra_sdk_fee::Fee;
use penumbra_sdk_keys::{test_keys, Address};
use penumbra_sdk_num::Amount;
use penumbra_sdk_sct::component::source::SourceContext;
use rand_core::SeedableRng;
use std::{ops::Deref, sync::Arc};

#[tokio::test]
/// Minimal reproduction of a bug in the DEX VCB swap flow tracking.
///
/// Overview: The DEX VCB was double-counting swap flows for a same asset
/// by computing:  `aggregate = (delta + aggregate) + aggregate`, instead
/// it should compute: `aggregate = delta + aggregate`.
/// This bug was fixed in #4643.
async fn dex_vcb_tracks_multiswap() -> anyhow::Result<()> {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1776);
    let storage = TempStorage::new_with_penumbra_prefixes()
        .await?
        .apply_default_genesis()
        .await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    // Create the first swap:
    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
    let trading_pair = TradingPair::new(gm.id(), gn.id());

    let delta_1 = Amount::from(100_000u64);
    let delta_2 = Amount::from(0u64);
    let fee = Fee::default();
    let claim_address: Address = test_keys::ADDRESS_0.deref().clone();
    let plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1, delta_2, fee, claim_address);

    let swap_plan = SwapPlan::new(&mut rng, plaintext.clone());
    let swap_one = swap_plan.swap(&test_keys::FULL_VIEWING_KEY);

    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    swap_one.check_and_execute(&mut state_tx).await?;

    // Observe the DEX VCB has been credited:
    let gm_vcb_amount = state_tx
        .get_dex_vcb_for_asset(&gm.id())
        .await?
        .expect("we just accumulated a swap");
    assert_eq!(
        gm_vcb_amount,
        100_000u128.into(),
        "the DEX VCB does not contain swap 1"
    );

    // Let's add another swap:
    let swap_two = swap_one.clone();
    swap_two.check_and_execute(&mut state_tx).await?;
    let gm_vcb_amount = state_tx
        .get_dex_vcb_for_asset(&gm.id())
        .await?
        .expect("we accumulated two swaps");
    assert_eq!(
        gm_vcb_amount,
        200_000u128.into(),
        "the DEX VCB does not contain swap 2"
    );

    Ok(())
}
