use {
    self::common::BuilderExt,
    anyhow::anyhow,
    cnidarium::TempStorage,
    common::TempStorageExt as _,
    penumbra_sdk_app::{
        genesis::{self, AppState},
        server::consensus::Consensus,
    },
    penumbra_sdk_asset::{Value, STAKING_TOKEN_ASSET_ID},
    penumbra_sdk_auction::{
        auction::{
            dutch::{ActionDutchAuctionEnd, ActionDutchAuctionSchedule, DutchAuctionDescription},
            AuctionNft,
        },
        component::AuctionStoreRead,
        StateReadExt as _,
    },
    penumbra_sdk_keys::test_keys,
    penumbra_sdk_mock_client::MockClient,
    penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_proto::DomainType,
    penumbra_sdk_shielded_pool::{Note, OutputPlan, SpendPlan},
    penumbra_sdk_transaction::{
        memo::MemoPlaintext, plan::MemoPlan, ActionPlan, TransactionParameters, TransactionPlan,
    },
    rand_core::OsRng,
    std::{ops::Deref, str::FromStr},
    tap::Tap,
    tracing::{error_span, info, Instrument},
    tracing_subscriber::filter::EnvFilter,
};

mod common;

#[tokio::test]
/// Minimal reproduction of `v0.75.0` auction VCB bug.
///
/// Overview: In some cases, ending an auction early might cause
/// corruption in the VCB because recalled LP position reserves are
/// not properly credited to the auction component.
///
/// Conditions: The Dutch auction is active at `seq=0` and has a deployed LP.
///
/// Trigger: The user submits a `ActionDutchAuctionEnd` to end the auction early.
///
/// Walkthrough:
/// To close the auction, the deployed liquidity position needs to be recalled.
/// This is implemented by `DutchAuctionManager::end_auction_by_id`, previously
/// the method omitted to credit the auction component's value circuit-breaker.
/// This can potentially lead to a subsequent chain halt, because the auction
/// component has an incorrect view of its own balance.
async fn app_can_reproduce_tesnet_75_vcb_close() -> anyhow::Result<()> {
    let guard = {
        let filter = EnvFilter::default()
            .add_directive(
                "penumbra_auction=trace"
                    .parse()
                    .expect("we only write valid code :)"),
            )
            .add_directive(
                "penumbra_dex=debug"
                    .parse()
                    .expect("we only write valid code :)"),
            );

        common::set_tracing_subscriber_with_env_filter(filter)
    };
    let storage = TempStorage::new_with_penumbra_prefixes().await?;
    let app_state = AppState::Content(
        genesis::Content::default().with_chain_id(TestNode::<()>::CHAIN_ID.to_string()),
    );

    let mut node = {
        let consensus = Consensus::new(storage.as_ref().clone());
        TestNode::builder()
            .single_validator()
            .with_penumbra_auto_app_state(app_state)?
            .init_chain(consensus)
            .await
    }?;

    let mut client = MockClient::new(test_keys::SPEND_KEY.clone())
        .with_sync_to_storage(&storage)
        .await?
        .tap(|c| info!(client.notes = %c.notes.len(), "mock client synced to test storage"));

    let input = Value {
        asset_id: *STAKING_TOKEN_ASSET_ID,
        amount: 1_000_000u128.into(),
    };

    let max_output = Value::from_str("100gm")?;
    let min_output = Value::from_str("1gm")?;

    let dutch_auction_description = DutchAuctionDescription {
        input,
        output_id: max_output.asset_id,
        max_output: max_output.amount,
        min_output: min_output.amount,
        start_height: 50,
        end_height: 100,
        step_count: 50,
        nonce: [0u8; 32],
    };

    let schedule_plan = ActionDutchAuctionSchedule {
        description: dutch_auction_description.clone(),
    };
    let auction_id = dutch_auction_description.id();

    let note = client
        .notes
        .values()
        .cloned()
        .filter(|note| note.asset_id() == *STAKING_TOKEN_ASSET_ID)
        .next()
        .ok_or_else(|| anyhow!("mock client had no note"))?;

    let spend_note: ActionPlan = SpendPlan::new(
        &mut rand_core::OsRng,
        note.clone(),
        client.position(note.commit()).expect("note is in SCT"),
    )
    .into();

    let change = OutputPlan::new(
        &mut OsRng,
        Value {
            asset_id: *STAKING_TOKEN_ASSET_ID,
            amount: 999_000_000u128.into(),
        },
        test_keys::ADDRESS_0.deref().clone(),
    );

    let nft_auction_open = AuctionNft::new(auction_id, 0);

    let nft_open_output_note = OutputPlan::new(
        &mut OsRng,
        Value {
            asset_id: nft_auction_open.asset_id(),
            amount: 1u128.into(),
        },
        test_keys::ADDRESS_0.deref().clone(),
    );

    let actions = vec![
        schedule_plan.into(),
        spend_note.into(),
        change.into(),
        nft_open_output_note.clone().into(),
    ];

    let plan = TransactionPlan {
        memo: Some(MemoPlan::new(
            &mut OsRng,
            MemoPlaintext::blank_memo(test_keys::ADDRESS_0.deref().clone()),
        )),
        actions,
        detection_data: None,
        transaction_parameters: TransactionParameters {
            chain_id: TestNode::<()>::CHAIN_ID.to_string(),
            ..Default::default()
        },
    }
    .with_populated_detection_data(OsRng, Default::default());

    let tx = client.witness_auth_build(&plan).await?;
    node.block()
        .add_tx(tx.encode_to_vec())
        .execute()
        .instrument(error_span!("schedule a dutch auction"))
        .await?;

    node.fast_forward(51).await?;
    let post_execution = storage.latest_snapshot();

    let auction_state = post_execution.get_dutch_auction_by_id(auction_id).await?;
    assert!(auction_state.is_some(), "the chain should have recorded some auction state associated with the description we provided");

    let action_end_auction = ActionDutchAuctionEnd { auction_id };
    let nft_auction_closed = AuctionNft::new(auction_id, 1);

    client.sync_to_latest(post_execution.clone()).await?;

    // Show that the client now has a single note for some delegation tokens.
    let nft_open_note: Note = {
        let mut notes: Vec<_> = client
            .notes_by_asset(nft_auction_open.asset_id())
            .cloned()
            .collect();
        assert_eq!(
            notes.len(),
            1,
            "we have exactly one note for the opened auction nft"
        );
        notes.pop().unwrap()
    };

    let nft_opened_spend_note: ActionPlan = SpendPlan::new(
        &mut rand_core::OsRng,
        nft_open_note.clone(),
        client
            .position(nft_open_note.commit())
            .expect("note is tracked in sct"),
    )
    .into();

    let nft_closed_output_note = OutputPlan::new(
        &mut OsRng,
        Value {
            asset_id: nft_auction_closed.asset_id(),
            amount: 1u128.into(),
        },
        test_keys::ADDRESS_0.deref().clone(),
    )
    .into();

    let actions = vec![
        nft_opened_spend_note.into(),
        action_end_auction.into(),
        nft_closed_output_note,
    ];

    let plan = TransactionPlan {
        memo: Some(MemoPlan::new(
            &mut OsRng,
            MemoPlaintext::blank_memo(test_keys::ADDRESS_0.deref().clone()),
        )),
        actions,
        detection_data: None,
        transaction_parameters: TransactionParameters {
            chain_id: TestNode::<()>::CHAIN_ID.to_string(),
            ..Default::default()
        },
    }
    .with_populated_detection_data(OsRng, Default::default());

    let tx = client.witness_auth_build(&plan).await?;
    tracing::info!("closing the auction");
    node.block()
        .add_tx(tx.encode_to_vec())
        .execute()
        .instrument(error_span!("end a dutch auction"))
        .await?;

    tracing::info!("fast-forwarding by one block");
    node.fast_forward(1).await?;
    let post_execution = storage.latest_snapshot();

    let new_auction_state = post_execution
        .get_dutch_auction_by_id(auction_id)
        .await?
        .expect("we established that the auction state exists");
    assert_eq!(new_auction_state.state.sequence, 1);

    // Assert that the auction VCB contains the right reserves:
    let auction_vcb_staking_token = post_execution
        .get_auction_value_balance_for(&STAKING_TOKEN_ASSET_ID)
        .await;
    assert_eq!(auction_vcb_staking_token, 1_000_000u128.into());

    Ok(())
        .tap(|_| drop(node))
        .tap(|_| drop(storage))
        .tap(|_| drop(guard))
}
