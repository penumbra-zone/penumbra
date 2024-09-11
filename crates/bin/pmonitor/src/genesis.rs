use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use penumbra_app::genesis::AppState;
use penumbra_asset::STAKING_TOKEN_ASSET_ID;
use penumbra_keys::FullViewingKey;
use penumbra_num::Amount;
use penumbra_shielded_pool::Note;
use penumbra_stake::{
    rate::{BaseRateData, RateData},
    DelegationToken,
};
use penumbra_tct::StateCommitment;

#[derive(Debug, Clone)]
pub struct FilteredGenesisBlock {
    // Notes per FVK
    pub notes: BTreeMap<String, BTreeMap<StateCommitment, Note>>,
    // UM-equivalent balances per FVK
    pub balances: BTreeMap<String, Amount>,
}

/// Scanning of the genesis `CompactBlock` with a list of FVKs to determine the
/// initial balances of the relevant addresses.
///
/// Assumption: There are no swaps or nullifiers in the genesis block.
pub fn scan_genesis_block(
    genesis_app_state: AppState,
    fvks: Vec<FullViewingKey>,
) -> anyhow::Result<FilteredGenesisBlock> {
    let mut notes = BTreeMap::new();
    let mut balances = BTreeMap::new();

    let genesis_data = genesis_app_state
        .content()
        .expect("genesis app state should have content");
    // We'll use the allocations from the genesis state.
    let shielded_pool_content = &genesis_data.shielded_pool_content;

    // Calculate the rate data for each validator in the initial validator set.
    let base_rate = BaseRateData {
        epoch_index: 0,
        base_reward_rate: 0u128.into(),
        base_exchange_rate: 1_0000_0000u128.into(),
    };
    let rate_data_map: HashMap<DelegationToken, RateData> = genesis_data
        .stake_content
        .validators
        .iter()
        .map(|validator| {
            let identity_key = validator
                .identity_key
                .clone()
                .expect("identity key should be present")
                .try_into()
                .expect("should be a valid identity key");
            let rate_data = RateData {
                identity_key,
                validator_reward_rate: 0u128.into(),
                validator_exchange_rate: base_rate.base_exchange_rate,
            };
            (DelegationToken::from(identity_key), rate_data)
        })
        .collect();

    // We proceed one FVK at a time.
    for fvk in fvks {
        let mut notes_for_this_fvk = BTreeMap::new();
        for allocation in &shielded_pool_content.allocations {
            if fvk.incoming().views_address(&allocation.address) {
                let note =
                    Note::from_allocation(allocation.clone()).expect("should be a valid note");
                notes_for_this_fvk.insert(note.commit(), note.clone());

                // Balance is expected to be in the staking or delegation token
                let allocation_value = allocation.value();
                if allocation_value.asset_id == *STAKING_TOKEN_ASSET_ID {
                    balances
                        .entry(fvk.to_string())
                        .and_modify(|existing_amount| *existing_amount += allocation.amount())
                        .or_insert(allocation.amount());
                } else if let Ok(delegation_token) =
                    DelegationToken::from_str(&allocation_value.asset_id.to_string())
                {
                    // We need to convert the amount to the UM-equivalent amount
                    let rate_data = rate_data_map
                        .get(&delegation_token)
                        .expect("should be rate data for each validator");
                    let um_equivalent_balance = rate_data.unbonded_amount(allocation.amount());

                    balances
                        .entry(fvk.to_string())
                        .and_modify(|existing_amount| *existing_amount += um_equivalent_balance)
                        .or_insert(um_equivalent_balance);
                } else {
                    tracing::warn!(
                        "ignoring note with unrecognized asset id: {}",
                        allocation_value.asset_id
                    );
                }
            }
        }

        // Save all the notes for this FVK, and continue.
        notes.insert(fvk.to_string(), notes_for_this_fvk);
    }

    // Construct filtered genesis block with allocations
    let result = FilteredGenesisBlock { notes, balances };

    Ok(result)
}
