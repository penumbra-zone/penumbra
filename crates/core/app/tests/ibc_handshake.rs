use {
    anyhow::anyhow,
    common::ibc_tests::{MockRelayer, TestNodeWithIBC, ValidatorKeys},
    ibc_proto::ibc::core::channel::v1::Channel,
    ibc_types::{
        core::{
            channel::{
                msgs::MsgRecvPacket, packet::Sequence, ChannelId, Packet, PortId, TimeoutHeight,
            },
            client::Height,
        },
        timestamp::Timestamp,
    },
    once_cell::sync::Lazy,
    penumbra_asset::{asset::Cache, Value},
    penumbra_fee::component::StateReadExt as _,
    penumbra_keys::keys::AddressIndex,
    penumbra_num::Amount,
    penumbra_proto::DomainType,
    penumbra_shielded_pool::{
        component::Ics20Transfer, Ics20Withdrawal, OutputPlan, Spend, SpendPlan,
    },
    penumbra_stake::state_key::chain,
    penumbra_transaction::{
        memo::MemoPlaintext, plan::MemoPlan, TransactionParameters, TransactionPlan,
    },
    penumbra_view::Planner,
    rand_core::OsRng,
    std::{
        str::FromStr,
        time::{Duration, SystemTime, UNIX_EPOCH},
    },
    tap::Tap as _,
};

/// The proof specs for the main store.
pub static MAIN_STORE_PROOF_SPEC: Lazy<Vec<ics23::ProofSpec>> =
    Lazy::new(|| vec![cnidarium::ics23_spec()]);

mod common;

/// Exercises that the IBC handshake succeeds.
#[tokio::test]
async fn ibc_handshake() -> anyhow::Result<()> {
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

    // Ensure chain A has balance to transfer
    let chain_a_client = relayer.chain_a_ibc.client().await?;
    let chain_b_client = relayer.chain_b_ibc.client().await?;
    let chain_a_note = chain_a_client
        .notes
        .values()
        .cloned()
        .next()
        .ok_or_else(|| anyhow!("mock client had no note"))?;

    // Get the balance of that asset on chain A
    let pretransfer_balance_a: Amount = chain_a_client
        .notes_by_asset(chain_a_note.asset_id())
        .map(|n| n.value().amount)
        .sum();

    // Get the balance of that asset on chain B
    let pretransfer_balance_b: Amount = chain_b_client
        .notes_by_asset(chain_a_note.asset_id())
        .map(|n| n.value().amount)
        .sum();

    // We will transfer 50% of the `chain_a_note`'s value to the same address on chain B
    let transfer_value = Value {
        amount: (chain_a_note.amount().value() / 2).into(),
        asset_id: chain_a_note.asset_id(),
    };

    println!("transferring value: {:?}", transfer_value);
    // Prepare and perform the transfer from chain A to chain B
    let destination_chain_address = chain_b_client.fvk.payment_address(AddressIndex::new(0)).0;
    let asset_cache = Cache::with_known_assets();
    let denom = asset_cache
        .get(&transfer_value.asset_id)
        .expect("asset ID should exist in asset cache")
        .clone();
    let amount = transfer_value.amount;
    // TODO: test timeouts
    // For this sunny path test, we'll set the timeouts very far in the future
    let timeout_height = Height {
        revision_height: 1_000_000,
        revision_number: 0,
    };
    // get the current time on the local machine
    let current_time_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64;

    // add 2 days to current time
    let mut timeout_time = current_time_ns + 1.728e14 as u64;

    // round to the nearest 10 minutes
    timeout_time += 600_000_000_000 - (timeout_time % 600_000_000_000);

    let return_address = chain_a_client
        .fvk
        // TODO: use of OsRng here makes tests non-deterministic. the mock relayer
        // is otherwise deterministic.
        .ephemeral_address(OsRng, AddressIndex::new(0))
        .0;
    let withdrawal = Ics20Withdrawal {
        destination_chain_address: destination_chain_address.to_string(),
        denom,
        amount,
        timeout_height,
        timeout_time,
        return_address,
        // TODO: this is fine to hardcode for now but should ultimately move
        // to the mock relayer and be based on the handshake
        source_channel: ChannelId::from_str("channel-0")?,
        // Penumbra <-> Penumbra so false
        use_compat_address: false,
    };
    // There will need to be `Spend` and `Output` actions
    // within the transaction in order for it to balance
    let spend_plan = SpendPlan::new(
        &mut rand_core::OsRng,
        chain_a_note.clone(),
        chain_a_client
            .position(chain_a_note.commit())
            .expect("note should be in mock client's tree"),
    );
    let output_plan = OutputPlan::new(
        &mut rand_core::OsRng,
        // half the note is being withdrawn, so we can use `transfer_value` both for the withdrawal action
        // and the change output
        transfer_value.clone(),
        chain_a_client.fvk.payment_address(AddressIndex::new(0)).0,
    );

    let plan = {
        let ics20_msg = withdrawal.into();
        TransactionPlan {
            actions: vec![ics20_msg, spend_plan.into(), output_plan.into()],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: Some(MemoPlan::new(
                &mut OsRng,
                MemoPlaintext::blank_memo(
                    chain_a_client.fvk.payment_address(AddressIndex::new(0)).0,
                ),
            )),
            detection_data: None, // We'll set this automatically below
            transaction_parameters: TransactionParameters {
                chain_id: relayer.chain_a_ibc.chain_id.clone(),
                ..Default::default()
            },
        }
        .with_populated_detection_data(OsRng, Default::default())
    };
    let tx = relayer
        .chain_a_ibc
        .client()
        .await?
        .witness_auth_build(&plan)
        .await?;

    let events = relayer
        .chain_a_ibc
        .node
        .block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .await?;
    relayer._sync_chains().await?;

    // Now that the withdrawal has been processed on Chain A, the relayer
    // tells chain B to process the transfer. It does this by forwarding a
    // MsgRecvPacket to chain B.
    // let msg_recv_packet = MsgRecvPacket { packet: todo!(), proof_commitment_on_a: todo!(), proof_height_on_a: todo!(), signer: todo!() }};
    // The relayer needs to extract the event that chain A emitted:
    for event in events {
        if event.kind == "send_packet" {
            let mut packet_data_hex;
            let mut sequence;
            let mut port_on_a;
            let mut chan_on_a;
            let mut port_on_b;
            let mut chan_on_b;
            let mut timeout_height_on_b;
            let mut timeout_timestamp_on_b;
            for attr in event.attributes {
                match attr.key.as_str() {
                    "packet_data_hex" => packet_data_hex = attr.value,
                    "packet_sequence" => sequence = attr.value,
                    "packet_src_port" => port_on_a = attr.value,
                    "packet_src_channel" => chan_on_a = attr.value,
                    "packet_dst_port" => port_on_b = attr.value,
                    "packet_dst_channel" => chan_on_b = attr.value,
                    "packet_timeout_height" => timeout_height_on_b = attr.value,
                    "packet_timeout_timestamp" => timeout_timestamp_on_b = attr.value,
                    _ => (),
                }
            }
            let msg_recv_packet = MsgRecvPacket {
                packet: Packet {
                    sequence: Sequence::from_str(&sequence)?,
                    port_on_a: PortId::from_str(&port_on_a)?,
                    chan_on_a: ChannelId::from_str(&chan_on_a)?,
                    port_on_b: PortId::from_str(&port_on_b)?,
                    chan_on_b: ChannelId::from_str(&chan_on_b)?,
                    data: hex::decode(packet_data_hex)?,
                    timeout_height_on_b: TimeoutHeight::from_str(&timeout_height_on_b)?,
                    timeout_timestamp_on_b: Timestamp::from_str(&timeout_timestamp_on_b)?,
                },
                proof_commitment_on_a: e.proof_commitment,
                proof_height_on_a: e.proof_height,
                signer: e.signer,
            };
            // relayer
            //     .chain_b_ibc
            //     .node
            //     .block()
            //     .with_data(vec![msg_recv_packet.encode_to_vec()])
            //     .execute()
            //     .await?;
            break;
        }
    }

    Ok(()).tap(|_| drop(relayer)).tap(|_| drop(guard))
}
