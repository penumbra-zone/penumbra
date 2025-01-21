//! Logic for inspecting the [CompactBlock] at genesis of the target chain.
//! Used to compute balances for tracked FVKs at genesis time. The initial genesis balance is
//! stored in the `pmonitor` config file, so that audit actions can reference it.
use std::{collections::BTreeMap, str::FromStr};

use penumbra_sdk_asset::STAKING_TOKEN_ASSET_ID;
use penumbra_sdk_compact_block::{CompactBlock, StatePayload};
use penumbra_sdk_keys::FullViewingKey;
use penumbra_sdk_num::Amount;
use penumbra_sdk_shielded_pool::{Note, NotePayload};
use penumbra_sdk_stake::{
    rate::{BaseRateData, RateData},
    DelegationToken,
};
use penumbra_sdk_tct::StateCommitment;

#[derive(Debug, Clone)]
pub struct FilteredGenesisBlock {
    // Notes per FVK
    #[allow(dead_code)]
    pub notes: BTreeMap<String, BTreeMap<StateCommitment, Note>>,
    // UM-equivalent balances per FVK
    pub balances: BTreeMap<String, Amount>,
}

/// Scanning of the genesis `CompactBlock` with a list of FVKs to determine the
/// initial balances of the relevant addresses.
///
/// Assumption: There are no swaps or nullifiers in the genesis block.
pub async fn scan_genesis_block(
    CompactBlock {
        height,
        state_payloads,
        ..
    }: CompactBlock,
    fvks: Vec<FullViewingKey>,
) -> anyhow::Result<FilteredGenesisBlock> {
    assert_eq!(height, 0);

    let mut notes = BTreeMap::new();
    let mut balances = BTreeMap::new();

    // Calculate the rate data for each validator in the initial validator set.
    let base_rate = BaseRateData {
        epoch_index: 0,
        base_reward_rate: 0u128.into(),
        base_exchange_rate: 1_0000_0000u128.into(),
    };

    // We proceed one FVK at a time.
    for fvk in fvks {
        // Trial-decrypt a note with our a specific viewing key
        let trial_decrypt_note =
            |note_payload: NotePayload| -> tokio::task::JoinHandle<Option<Note>> {
                let fvk2 = fvk.clone();
                tokio::spawn(async move { note_payload.trial_decrypt(&fvk2) })
            };

        // Trial-decrypt the notes in this block, keeping track of the ones that were meant for the FVK
        // we're monitoring.
        let mut note_decryptions = Vec::new();

        // We only care about notes, so we're ignoring swaps and rolled-up commitments.
        for payload in state_payloads.iter() {
            if let StatePayload::Note { note, .. } = payload {
                note_decryptions.push(trial_decrypt_note((**note).clone()));
            }
        }

        let mut notes_for_this_fvk = BTreeMap::new();
        for decryption in note_decryptions {
            if let Some(note) = decryption
                .await
                .expect("able to join tokio note decryption handle")
            {
                notes_for_this_fvk.insert(note.commit(), note.clone());

                // Balance is expected to be in the staking or delegation token
                let note_value = note.value();
                if note_value.asset_id == *STAKING_TOKEN_ASSET_ID {
                    balances
                        .entry(fvk.to_string())
                        .and_modify(|existing_amount| *existing_amount += note.amount())
                        .or_insert(note.amount());
                } else if let Ok(delegation_token) =
                    DelegationToken::from_str(&note_value.asset_id.to_string())
                {
                    // We need to convert the amount to the UM-equivalent amount
                    let rate_data = RateData {
                        identity_key: delegation_token.validator(),
                        validator_reward_rate: 0u128.into(),
                        validator_exchange_rate: base_rate.base_exchange_rate,
                    };
                    let um_equivalent_balance = rate_data.unbonded_amount(note.amount());

                    balances
                        .entry(fvk.to_string())
                        .and_modify(|existing_amount| *existing_amount += um_equivalent_balance)
                        .or_insert(um_equivalent_balance);
                } else {
                    tracing::warn!(
                        "ignoring note with unknown asset id: {}",
                        note_value.asset_id
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
