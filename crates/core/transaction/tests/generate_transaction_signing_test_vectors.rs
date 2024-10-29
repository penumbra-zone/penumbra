use std::fs::File;

use decaf377::{Fq, Fr};
use decaf377_rdsa::{SigningKey, SpendAuth, VerificationKey, VerificationKeyBytes};
use ed25519_consensus::SigningKey as Ed25519SigningKey;
use ibc_types::core::{
    channel::{msgs::MsgRecvPacket, packet::Sequence, ChannelId, Packet, PortId},
    client::Height,
    commitment::MerkleProof,
};
use ibc_types::timestamp::Timestamp;
use penumbra_asset::asset::Id;
use penumbra_dex::{
    swap::{SwapPlaintext, SwapPlan},
    swap_claim::SwapClaimPlan,
    BatchSwapOutputData, TradingPair,
};
use penumbra_fee::Fee;
use penumbra_ibc::IbcRelay;
use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_keys::{Address, FullViewingKey};
use penumbra_num::Amount;
use penumbra_proto::DomainType;
use penumbra_sct::epoch::Epoch;
use penumbra_shielded_pool::{Note, OutputPlan, SpendPlan};
use penumbra_stake::{
    validator, validator::Definition, Delegate, FundingStreams, GovernanceKey, IdentityKey,
    Penalty, Undelegate, UndelegateClaimPlan,
};
use penumbra_transaction::{ActionPlan, TransactionParameters, TransactionPlan};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::{Config, TestRunner};
use rand_core::OsRng;
use std::io::Write;
use tendermint;

fn amount_strategy() -> impl Strategy<Value = Amount> {
    let inner_uint_range = 0u128..1_000_000_000_000_000_000u128;
    inner_uint_range.prop_map(|uint| Amount::from_le_bytes(uint.to_le_bytes()))
}

fn asset_id_strategy() -> impl Strategy<Value = Id> {
    Just(*penumbra_asset::STAKING_TOKEN_ASSET_ID)
}

fn value_strategy() -> impl Strategy<Value = penumbra_asset::Value> {
    (asset_id_strategy(), amount_strategy())
        .prop_map(|(asset_id, amount)| penumbra_asset::Value { amount, asset_id })
}

fn address_strategy() -> impl Strategy<Value = Address> {
    // normally we would use address::dummy, but this seems to not work properly
    // for some reason (invalid key errors on computing effecthash.)
    prop::strategy::LazyJust::new(|| {
        let seed_phrase = SeedPhrase::generate(&mut OsRng);
        let sk = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let addr = sk.full_viewing_key().payment_address(0u32.into()).0;

        addr
    })
}

fn note_strategy(addr: Address) -> impl Strategy<Value = Note> {
    value_strategy().prop_map(move |value| Note::generate(&mut OsRng, &addr, value))
}

fn spend_plan_strategy(fvk: &FullViewingKey) -> impl Strategy<Value = SpendPlan> {
    let tct_strategy = any::<penumbra_tct::Position>();
    let note_strategy = note_strategy(fvk.incoming().payment_address(0u32.into()).0);

    (tct_strategy, note_strategy)
        .prop_map(|(tct_pos, note)| SpendPlan::new(&mut OsRng, note, tct_pos))
}

fn output_plan_strategy() -> impl Strategy<Value = OutputPlan> {
    (value_strategy(), address_strategy())
        .prop_map(|(value, address)| OutputPlan::new(&mut OsRng, value, address))
}

fn identity_key_strategy() -> impl Strategy<Value = IdentityKey> {
    let rand_bytes = prop::array::uniform32(any::<u8>());

    rand_bytes.prop_map(|vk_bytes| IdentityKey(VerificationKeyBytes::<SpendAuth>::from(vk_bytes)))
}

fn delegate_plan_strategy() -> impl Strategy<Value = Delegate> {
    let epoch_index_strategy = 0..10000u64;
    let unbonded_amount_strategy = amount_strategy();
    let delegation_amount_strategy = amount_strategy();

    (
        identity_key_strategy(),
        epoch_index_strategy,
        unbonded_amount_strategy,
        delegation_amount_strategy,
    )
        .prop_map(
            |(validator_identity, epoch_index, unbonded_amount, delegation_amount)| Delegate {
                validator_identity,
                epoch_index,
                unbonded_amount,
                delegation_amount,
            },
        )
}

fn undelegate_plan_strategy() -> impl Strategy<Value = Undelegate> {
    let epoch_index_strategy = 0..10000u64;
    let unbonded_amount_strategy = amount_strategy();
    let delegation_amount_strategy = amount_strategy();
    (
        identity_key_strategy(),
        epoch_index_strategy,
        unbonded_amount_strategy,
        delegation_amount_strategy,
    )
        .prop_map(
            |(validator_identity, epoch_index, unbonded_amount, delegation_amount)| Undelegate {
                validator_identity,
                from_epoch: Epoch {
                    index: epoch_index,
                    start_height: epoch_index,
                },
                unbonded_amount,
                delegation_amount,
            },
        )
}

fn undelegate_claim_plan_strategy() -> impl Strategy<Value = UndelegateClaimPlan> {
    let penalty_bps = 0..100u64;
    let unbonding_start_height_strategy = 1000..100000u64;
    (
        identity_key_strategy(),
        penalty_bps,
        amount_strategy(),
        unbonding_start_height_strategy,
    )
        .prop_map(
            |(validator_identity, penalty_bps, unbonding_amount, unbonding_start_height)| {
                UndelegateClaimPlan {
                    validator_identity,
                    penalty: Penalty::from_bps(penalty_bps),
                    unbonding_amount,
                    balance_blinding: Fr::rand(&mut OsRng),
                    proof_blinding_r: Fq::rand(&mut OsRng),
                    proof_blinding_s: Fq::rand(&mut OsRng),
                    unbonding_start_height,
                }
            },
        )
}

fn signing_key_strategy() -> impl Strategy<Value = SigningKey<SpendAuth>> {
    prop::strategy::LazyJust::new(|| SigningKey::<SpendAuth>::new(OsRng))
}

fn consensus_secret_key_strategy() -> impl Strategy<Value = Ed25519SigningKey> {
    prop::strategy::LazyJust::new(|| Ed25519SigningKey::new(OsRng))
}

fn validator_strategy() -> impl Strategy<Value = (validator::Validator, SigningKey<SpendAuth>)> {
    (signing_key_strategy(), consensus_secret_key_strategy()).prop_map(
        move |(new_validator_id_sk, new_validator_consensus_sk)| {
            let new_validator_id = IdentityKey(VerificationKey::from(&new_validator_id_sk).into());
            let new_validator_consensus = new_validator_consensus_sk.verification_key();
            (
                validator::Validator {
                    identity_key: new_validator_id.clone(),
                    consensus_key: tendermint::PublicKey::from_raw_ed25519(
                        &new_validator_consensus.to_bytes(),
                    )
                    .expect("consensus key is valid"),
                    governance_key: GovernanceKey(new_validator_id_sk.into()),
                    enabled: true,
                    sequence_number: 0,
                    name: "test validator".to_string(),
                    website: String::default(),
                    description: String::default(),
                    funding_streams: FundingStreams::default(),
                },
                new_validator_id_sk,
            )
        },
    )
}

fn validator_definition_strategy() -> impl Strategy<Value = Definition> {
    (validator_strategy()).prop_map(|(new_validator, new_validator_id_sk)| {
        let bytes = new_validator.encode_to_vec();
        let auth_sig = new_validator_id_sk.sign(OsRng, &bytes);
        Definition {
            validator: new_validator,
            auth_sig,
        }
    })
}

fn swap_plaintext_strategy() -> impl Strategy<Value = SwapPlaintext> {
    (
        amount_strategy(),
        amount_strategy(),
        asset_id_strategy(),
        asset_id_strategy(),
        address_strategy(),
    )
        .prop_map(|(delta_1_i, delta_2_i, asset_1, asset_2, claim_address)| {
            let trading_pair = TradingPair::new(asset_1, asset_2);
            SwapPlaintext::new(
                &mut OsRng,
                trading_pair,
                delta_1_i,
                delta_2_i,
                Fee::from_staking_token_amount(0u64.into()),
                claim_address,
            )
        })
}

fn swap_plan_strategy() -> impl Strategy<Value = SwapPlan> {
    (swap_plaintext_strategy()).prop_map(|swap_plaintext| SwapPlan {
        proof_blinding_r: Fq::rand(&mut OsRng),
        proof_blinding_s: Fq::rand(&mut OsRng),
        swap_plaintext,
        fee_blinding: Fr::rand(&mut OsRng),
    })
}

fn batch_swap_output_data_strategy() -> impl Strategy<Value = BatchSwapOutputData> {
    // Represents a filled swap
    let delta_1 = (4001..2000000000u128).prop_map(Amount::from);
    let delta_2 = (4001..2000000000u128).prop_map(Amount::from);

    let lambda_1 = (2..2000u64).prop_map(Amount::from);
    let lambda_2 = (2..2000u64).prop_map(Amount::from);

    let unfilled_1 = (2..2000u64).prop_map(Amount::from);
    let unfilled_2 = (2..2000u64).prop_map(Amount::from);

    (
        delta_1,
        delta_2,
        lambda_1,
        lambda_2,
        unfilled_1,
        unfilled_2,
        asset_id_strategy(),
        asset_id_strategy(),
    )
        .prop_map(
            |(
                delta_1,
                delta_2,
                lambda_1,
                lambda_2,
                unfilled_1,
                unfilled_2,
                asset_id_1,
                asset_id_2,
            )| BatchSwapOutputData {
                delta_1,
                delta_2,
                lambda_1,
                lambda_2,
                unfilled_1,
                unfilled_2,
                height: 0u64.into(),
                trading_pair: TradingPair::new(asset_id_1, asset_id_2),
                sct_position_prefix: Default::default(),
            },
        )
}

fn swap_claim_plan_strategy() -> impl Strategy<Value = SwapClaimPlan> {
    (swap_plaintext_strategy(), batch_swap_output_data_strategy()).prop_map(
        |(swap_plaintext, output_data)| SwapClaimPlan {
            swap_plaintext,
            position: penumbra_tct::Position::from(0u64),
            output_data,
            epoch_duration: 1000u64,
            proof_blinding_r: Fq::rand(&mut OsRng),
            proof_blinding_s: Fq::rand(&mut OsRng),
        },
    )
}

fn sequence_strategy() -> impl Strategy<Value = Sequence> {
    (4001..2000000000u64).prop_map(Sequence)
}

fn ibc_action_strategy() -> impl Strategy<Value = IbcRelay> {
    (
        sequence_strategy(),
        0..1000000000u64,
        0..1000000000u64,
        address_strategy(),
    )
        .prop_map(|(sequence, revision_number, revision_height, src)| {
            IbcRelay::RecvPacket(MsgRecvPacket {
                packet: Packet {
                    sequence,
                    port_on_a: PortId::default(),
                    chan_on_a: ChannelId::default(),
                    port_on_b: PortId::default(),
                    chan_on_b: ChannelId::default(),
                    data: vec![],
                    timeout_height_on_b: ibc_types::core::channel::TimeoutHeight::At(
                        Height::new(revision_number, revision_height).expect("test value"),
                    ),
                    timeout_timestamp_on_b: Timestamp::now(),
                },
                proof_commitment_on_a: MerkleProof { proofs: vec![] },
                proof_height_on_a: Height::new(revision_number, revision_height)
                    .expect("test value"),
                signer: src.to_string(),
            })
        })
}

fn action_plan_strategy(fvk: &FullViewingKey) -> impl Strategy<Value = ActionPlan> {
    prop_oneof![
        spend_plan_strategy(fvk).prop_map(ActionPlan::Spend),
        output_plan_strategy().prop_map(ActionPlan::Output),
        delegate_plan_strategy().prop_map(ActionPlan::Delegate),
        undelegate_plan_strategy().prop_map(ActionPlan::Undelegate),
        undelegate_claim_plan_strategy().prop_map(ActionPlan::UndelegateClaim),
        validator_definition_strategy().prop_map(ActionPlan::ValidatorDefinition),
        swap_plan_strategy().prop_map(ActionPlan::Swap),
        swap_claim_plan_strategy().prop_map(ActionPlan::SwapClaim),
        ibc_action_strategy().prop_map(ActionPlan::IbcAction),
    ]
}

fn actions_vec_strategy(fvk: &FullViewingKey) -> impl Strategy<Value = Vec<ActionPlan>> {
    prop::collection::vec(action_plan_strategy(fvk), 2..5)
}

fn transaction_parameters_strategy() -> impl Strategy<Value = TransactionParameters> {
    let expiry_height = 0u64..10000000000u64;
    let chain_id = prop::string::string_regex(r"[a-z]+-[0-9]+").unwrap();
    let fee = value_strategy().prop_map(|fee_value| Fee(fee_value));

    (expiry_height, chain_id, fee).prop_map(|(expiry_height, chain_id, fee)| {
        TransactionParameters {
            expiry_height,
            chain_id,
            fee,
        }
    })
}

fn transaction_plan_strategy(fvk: &FullViewingKey) -> impl Strategy<Value = TransactionPlan> {
    (actions_vec_strategy(fvk), transaction_parameters_strategy()).prop_map(|(actions, params)| {
        TransactionPlan {
            actions,
            transaction_parameters: params,
            detection_data: None,
            memo: None,
        }
    })
}

#[test]
fn generate_transaction_signing_test_vectors() {
    let mut runner = TestRunner::new(Config::default());
    let test_vectors_dir = "tests/signing_test_vectors";
    std::fs::create_dir_all(test_vectors_dir).expect("failed to create test vectors dir");

    let rng = OsRng;

    for i in 0..100 {
        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk = sk.full_viewing_key();
        let value_tree = transaction_plan_strategy(fvk)
            .new_tree(&mut runner)
            .expect("Failed to create new tree");
        let transaction_plan = value_tree.current();

        let json_plan = serde_json::to_string_pretty(&transaction_plan)
            .expect("should be able to json tx plan");

        let transaction_plan_encoded = transaction_plan.encode_to_vec();
        let effect_hash_hex = hex::encode(
            transaction_plan
                .effect_hash(fvk)
                .expect("should be able to compute effect hash")
                .0,
        );

        let json_file_path = format!("{}/transaction_plan_{}.json", test_vectors_dir, i);
        let proto_file_path = format!("{}/transaction_plan_{}.proto", test_vectors_dir, i);
        let hash_file_path = format!("{}/effect_hash_{}.txt", test_vectors_dir, i);

        let mut json_file = File::create(&json_file_path).expect("Failed to create JSON file");
        json_file
            .write_all(json_plan.as_bytes())
            .expect("Failed to write JSON file");
        let mut proto_file =
            File::create(&proto_file_path).expect("Failed to create Protobuf file");
        proto_file
            .write_all(&transaction_plan_encoded)
            .expect("Failed to write Protobuf file");

        // Write effect hash
        let mut hash_file = File::create(&hash_file_path).expect("Failed to create hash file");
        hash_file
            .write_all(effect_hash_hex.as_bytes())
            .expect("Failed to write hash file");
    }
}
