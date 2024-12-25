use {
    anyhow::anyhow,
    common::ibc_tests::{MockRelayer, TestNodeWithIBC, ValidatorKeys},
    once_cell::sync::Lazy,
    penumbra_sdk_asset::{asset::Cache, Value},
    penumbra_sdk_ibc::IbcToken,
    penumbra_sdk_num::Amount,
    std::time::Duration,
    tap::Tap as _,
};

/// The proof specs for the main store.
pub static MAIN_STORE_PROOF_SPEC: Lazy<Vec<ics23::ProofSpec>> =
    Lazy::new(|| vec![cnidarium::ics23_spec()]);

mod common;

/// Exercises that the IBC handshake succeeds, and that
/// funds can be sent between the two chains successfully,
/// without any testing of error conditions.
#[tokio::test]
async fn ics20_transfer_no_timeouts() -> anyhow::Result<()> {
    // Install a test logger, and acquire some temporary storage.
    let guard = common::set_tracing_subscriber();

    let block_duration = Duration::from_secs(5);
    // Fixed start times (both chains start at the same time to avoid unintended timeouts):
    let start_time_a = tendermint::Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z")?;

    // But chain B will be 39 blocks ahead of chain A, so offset chain A's
    // start time so they match:
    let start_time_b = start_time_a.checked_sub(39 * block_duration).unwrap();

    // Hardcoded keys for each chain for test reproducibility:
    let vkeys_a = ValidatorKeys::from_seed([0u8; 32]);
    let vkeys_b = ValidatorKeys::from_seed([1u8; 32]);
    let sk_a = vkeys_a.validator_cons_sk.ed25519_signing_key().unwrap();
    let sk_b = vkeys_b.validator_cons_sk.ed25519_signing_key().unwrap();

    let ska = ed25519_consensus::SigningKey::try_from(sk_a.as_bytes())?;
    let skb = ed25519_consensus::SigningKey::try_from(sk_b.as_bytes())?;
    let keys_a = (ska.clone(), ska.verification_key());
    let keys_b = (skb.clone(), skb.verification_key());

    // Set up some configuration for the two different chains we'll need to keep around.
    let mut chain_a_ibc = TestNodeWithIBC::new("a", start_time_a, keys_a).await?;
    let mut chain_b_ibc = TestNodeWithIBC::new("b", start_time_b, keys_b).await?;

    // The two chains can't IBC handshake during the first block, let's fast forward
    // them both a few.
    for _ in 0..3 {
        chain_a_ibc.node.block().execute().await?;
    }
    // Do them each a different # of blocks to make sure the heights don't get confused.
    for _ in 0..42 {
        chain_b_ibc.node.block().execute().await?;
    }

    // The chains should be at the same time:
    assert_eq!(chain_a_ibc.node.timestamp(), chain_b_ibc.node.timestamp());
    // But their block heights should be different:
    assert_ne!(
        chain_a_ibc.get_latest_height().await?,
        chain_b_ibc.get_latest_height().await?,
    );

    assert_eq!(
        chain_a_ibc.get_latest_height().await?.revision_height,
        chain_a_ibc.storage.latest_snapshot().version()
    );

    // The Relayer will handle IBC operations and manage state for the two test chains
    let mut relayer = MockRelayer {
        chain_a_ibc,
        chain_b_ibc,
    };

    // Perform the IBC connection and channel handshakes between the two chains.
    // TODO: some testing of failure cases of the handshake process would be good
    relayer.handshake().await?;

    // Grab the note that will be spent during the transfer.
    let chain_a_client = relayer.chain_a_ibc.client().await?;
    let chain_a_note = chain_a_client
        .notes
        .values()
        .cloned()
        .next()
        .ok_or_else(|| anyhow!("mock client had no note"))?;

    // Get the balance of that asset on chain A
    let pretransfer_balance_a: Amount = chain_a_client
        .spendable_notes_by_asset(chain_a_note.asset_id())
        .map(|n| n.value().amount)
        .sum();

    // Get the balance of that asset on chain B
    // The asset ID of the IBC transferred asset on chain B
    // needs to be computed.
    let asset_cache = Cache::with_known_assets();
    let denom = asset_cache
        .get(&chain_a_note.asset_id())
        .expect("asset ID should exist in asset cache")
        .clone();
    let ibc_token = IbcToken::new(
        &relayer.chain_b_ibc.channel_id,
        &relayer.chain_b_ibc.port_id,
        &denom.to_string(),
    );
    let chain_b_client = relayer.chain_b_ibc.client().await?;
    let pretransfer_balance_b: Amount = chain_b_client
        .spendable_notes_by_asset(ibc_token.id())
        .map(|n| n.value().amount)
        .sum();

    // We will transfer 50% of the `chain_a_note`'s value to the same address on chain B
    let transfer_value = Value {
        amount: (chain_a_note.amount().value() / 2).into(),
        asset_id: chain_a_note.asset_id(),
    };

    // Tell the relayer to process the transfer.
    // TODO: currently this just transfers 50% of the first note
    // but it'd be nice to have an API with a little more flexibility
    relayer.transfer_from_a_to_b().await?;

    // Transfer complete, validate the balances:
    let chain_a_client = relayer.chain_a_ibc.client().await?;
    let chain_b_client = relayer.chain_b_ibc.client().await?;
    let posttransfer_balance_a: Amount = chain_a_client
        .spendable_notes_by_asset(chain_a_note.asset_id())
        .map(|n| n.value().amount)
        .sum();

    let posttransfer_balance_b: Amount = chain_b_client
        .spendable_notes_by_asset(ibc_token.id())
        .map(|n| n.value().amount)
        .sum();

    assert!(posttransfer_balance_a < pretransfer_balance_a);
    assert!(posttransfer_balance_b > pretransfer_balance_b);
    assert_eq!(posttransfer_balance_b, transfer_value.amount);

    Ok(()).tap(|_| drop(relayer)).tap(|_| drop(guard))
}
