use std::collections::BTreeMap;

use anyhow::Result;
use comfy_table::{presets, Table};
use futures::TryStreamExt;
use penumbra_component::stake::validator;
use penumbra_crypto::{stake::DelegationToken, FullViewingKey, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_proto::client::v1alpha1::{
    oblivious_query_service_client::ObliviousQueryServiceClient, ValidatorInfoRequest,
};
use penumbra_view::ViewClient;
use tonic::transport::Channel;

#[derive(Debug, clap::Parser)]
pub struct StakedCmd {}

impl StakedCmd {
    pub fn offline(&self) -> bool {
        false
    }

    pub async fn exec(
        &self,
        full_viewing_key: &FullViewingKey,
        view_client: &mut impl ViewClient,
        oblivious_client: &mut ObliviousQueryServiceClient<Channel>,
    ) -> Result<()> {
        let client = oblivious_client;

        let asset_cache = view_client.assets().await?;

        let validators = client
            .validator_info(ValidatorInfoRequest {
                show_inactive: true,
                ..Default::default()
            })
            .await?
            .into_inner()
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<validator::Info>, _>>()?;

        let account_id = full_viewing_key.account_id();
        let notes = view_client
            .unspent_notes_by_asset_and_address(account_id)
            .await?;
        let mut total = 0u64;

        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        table.set_header(vec!["Name", "Value", "Exch. Rate", "Tokens"]);
        table
            .get_column_mut(1)
            .unwrap()
            .set_cell_alignment(comfy_table::CellAlignment::Right);

        for (asset_id, notes_by_address) in notes.iter() {
            let dt = if let Some(Ok(dt)) = asset_cache
                .get(asset_id)
                .map(|denom| DelegationToken::try_from(denom.clone()))
            {
                dt
            } else {
                continue;
            };

            let info = validators
                .iter()
                .find(|v| v.validator.identity_key == dt.validator())
                .unwrap();

            let delegation = Value {
                amount: notes_by_address
                    .values()
                    .flat_map(|notes| notes.iter().map(|n| u64::from(n.note.amount())))
                    .sum::<u64>()
                    .into(),
                asset_id: dt.id(),
            };

            let unbonded = Value {
                amount: info
                    .rate_data
                    .unbonded_amount(delegation.amount.into())
                    .into(),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            };

            let rate = info.rate_data.validator_exchange_rate as f64 / 1_0000_0000.0;

            table.add_row(vec![
                info.validator.name.clone(),
                unbonded.format(&asset_cache),
                format!("{rate:.4}"),
                delegation.format(&asset_cache),
            ]);

            total += u64::from(unbonded.amount);
        }

        let unbonded = Value {
            amount: notes
                .get(&*STAKING_TOKEN_ASSET_ID)
                .unwrap_or(&BTreeMap::default())
                .values()
                .flat_map(|notes| notes.iter().map(|n| u64::from(n.note.amount())))
                .sum::<u64>()
                .into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };

        total += u64::from(unbonded.amount);

        table.add_row(vec![
            "Unbonded Stake".to_string(),
            unbonded.format(&asset_cache),
            format!("{:.4}", 1.0),
            unbonded.format(&asset_cache),
        ]);

        let total = Value {
            amount: total.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };

        table.add_row(vec![
            "Total".to_string(),
            total.format(&asset_cache),
            String::new(),
            String::new(),
        ]);
        println!("{table}");

        Ok(())
    }
}
