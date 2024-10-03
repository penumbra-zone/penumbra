use penumbra_asset::asset::Id;
use penumbra_fee::Fee;
use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_keys::{Address, FullViewingKey};
use penumbra_num::Amount;
use penumbra_shielded_pool::{Note, OutputPlan, SpendPlan};
use penumbra_transaction::{ActionPlan, TransactionParameters, TransactionPlan};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::{Config, TestRunner};
use rand_core::OsRng;

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

fn action_plan_strategy(fvk: &FullViewingKey) -> impl Strategy<Value = ActionPlan> {
    prop_oneof![
        spend_plan_strategy(fvk).prop_map(ActionPlan::Spend),
        output_plan_strategy().prop_map(ActionPlan::Output),
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

fn main() {
    let mut runner = TestRunner::new(Config::default());

    let rng = OsRng;

    for _ in 0..100 {
        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk = sk.full_viewing_key();
        let value_tree = transaction_plan_strategy(fvk)
            .new_tree(&mut runner)
            .expect("Failed to create new tree");
        let transaction_plan = value_tree.current();

        println!("{:#?}", transaction_plan);
        println!("{:#?}", transaction_plan.effect_hash(fvk))
    }
}
