use std::collections::BTreeMap;

use anyhow::{anyhow, Context, Result};
use comfy_table::{presets, Table};
use futures::stream::TryStreamExt;
use penumbra_component::stake::{rate::RateData, validator};
use penumbra_crypto::{DelegationToken, IdentityKey, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_proto::client::oblivious::ValidatorInfoRequest;
use penumbra_view::ViewClient;
use penumbra_wallet::plan;
use rand_core::OsRng;

use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum StakeCmd {
    /// Deposit stake into a validator's delegation pool.
    Delegate {
        /// The identity key of the validator to delegate to.
        #[clap(long)]
        to: String,
        /// The amount of stake to delegate.
        amount: String,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[clap(long)]
        source: Option<u64>,
    },
    /// Withdraw stake from a validator's delegation pool.
    Undelegate {
        /// The amount of delegation tokens to undelegate.
        amount: String,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[clap(long)]
        source: Option<u64>,
    },
    /// Redelegate stake from one validator's delegation pool to another.
    Redelegate {
        /// The identity key of the validator to withdraw delegation from.
        #[clap(long)]
        from: String,
        /// The identity key of the validator to delegate to.
        #[clap(long)]
        to: String,
        /// The amount of stake to delegate.
        amount: String,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[clap(long)]
        source: Option<u64>,
    },
    /// Display this wallet's delegations and their value.
    Show,
    /// Display all of the validators participating in the chain.
    ListValidators {
        /// Whether to show validators that are not currently part of the consensus set.
        #[clap(short = 'i', long)]
        show_inactive: bool,
        /// Whether to show detailed validator info.
        #[clap(short, long)]
        detailed: bool,
    },
}

impl StakeCmd {
    pub fn needs_sync(&self) -> bool {
        true
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            StakeCmd::Delegate {
                to,
                amount,
                fee,
                source,
            } => {
                let unbonded_amount = {
                    let Value { amount, asset_id } = amount.parse::<Value>()?;
                    if asset_id != *STAKING_TOKEN_ASSET_ID {
                        return Err(anyhow!("staking can only be done with the staking token"));
                    }
                    amount
                };

                let to = to.parse::<IdentityKey>()?;

                let mut client = app.specific_client().await?;
                let rate_data: RateData = client
                    .next_validator_rate(tonic::Request::new(to.into()))
                    .await?
                    .into_inner()
                    .try_into()?;

                let plan = plan::delegate(
                    &app.fvk,
                    &mut app.view,
                    OsRng,
                    rate_data,
                    unbonded_amount,
                    *fee,
                    *source,
                )
                .await?;

                app.build_and_submit_transaction(plan).await?;
            }
            StakeCmd::Undelegate {
                amount,
                fee,
                source,
            } => {
                let (self_address, _dtk) = app
                    .fvk
                    .incoming()
                    .payment_address(source.unwrap_or(0).into());

                let delegation_value @ Value {
                    amount: _,
                    asset_id,
                } = amount.parse::<Value>()?;

                let delegation_token: DelegationToken = app
                    .view()
                    .assets()
                    .await?
                    .get(&asset_id)
                    .ok_or_else(|| anyhow::anyhow!("unknown asset id {}", asset_id))?
                    .clone()
                    .try_into()
                    .context("could not parse supplied denomination as a delegation token")?;

                let from = delegation_token.validator();

                let mut client = app.specific_client().await?;
                let rate_data: RateData = client
                    .next_validator_rate(tonic::Request::new(from.into()))
                    .await?
                    .into_inner()
                    .try_into()?;

                // first, split the input notes into exact change
                let split_plan = plan::send(
                    &app.fvk,
                    &mut app.view,
                    OsRng,
                    &[delegation_value],
                    *fee,
                    self_address,
                    *source,
                    None,
                )
                .await?;

                // find the note commitment corresponding to the delegation value within the split
                // plan, so that we can use it to create the undelegate plan
                let delegation_note_commitment = split_plan
                    .output_plans()
                    .find_map(|output| {
                        let note = output.output_note();
                        // grab the note commitment of whichever output in the spend plan has
                        // exactly the right amount and asset id, and is also addressed to us
                        if note.value() == delegation_value
                        // this check is not necessary currently, because we never construct
                        // undelegations to a different address than ourselves, but it's good to
                        // leave it in here so that if we ever change that invariant, it will fail
                        // here rather than after already executing the plan
                            && app.fvk.incoming().views_address(&output.dest_address)
                        {
                            Some(note.commit())
                        } else {
                            None
                        }
                    })
                    .expect("there must be an exact output for the amount we are expecting");

                // we submit the split transaction before building the undelegate plan, because we
                // need to await the note created by its output
                app.build_and_submit_transaction(split_plan).await?;

                // await the receipt of the exact note we wish to undelegate
                let delegation_notes = vec![
                    app.view
                        .await_note_by_commitment(app.fvk.hash(), delegation_note_commitment)
                        .await?,
                ];

                // now we can plan and submit an exact-change undelegation
                let undelegate_plan = plan::undelegate(
                    &app.fvk,
                    &mut app.view,
                    OsRng,
                    rate_data,
                    delegation_notes,
                    *fee,
                    *source,
                )
                .await?;

                // Pass None as the change to await, since the change will be quarantined, so we won't detect it.
                // But it's not spendable anyways, so we don't need to detect it.
                let tx = app.build_transaction(undelegate_plan).await?;
                app.submit_transaction(&tx, None).await?;
            }
            StakeCmd::Redelegate { .. } => {
                todo!()
            }
            StakeCmd::Show => {
                let mut client = app.oblivious_client().await?;

                let asset_cache = app.view().assets().await?;

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

                let fvk_hash = app.fvk.hash();
                let notes = app
                    .view()
                    .unspent_notes_by_asset_and_address(fvk_hash)
                    .await?;
                let mut total = 0;

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
                            .flat_map(|notes| notes.iter().map(|n| n.note.amount()))
                            .sum::<u64>(),
                        asset_id: dt.id(),
                    };

                    let unbonded = Value {
                        amount: info.rate_data.unbonded_amount(delegation.amount),
                        asset_id: *STAKING_TOKEN_ASSET_ID,
                    };

                    let rate = info.rate_data.validator_exchange_rate as f64 / 1_0000_0000.0;

                    table.add_row(vec![
                        info.validator.name.clone(),
                        unbonded.try_format(&asset_cache).unwrap(),
                        format!("{:.4}", rate),
                        delegation.try_format(&asset_cache).unwrap(),
                    ]);

                    total += unbonded.amount;
                }

                let unbonded = Value {
                    amount: notes
                        .get(&*STAKING_TOKEN_ASSET_ID)
                        .unwrap_or(&BTreeMap::default())
                        .values()
                        .flat_map(|notes| notes.iter().map(|n| n.note.amount()))
                        .sum::<u64>(),
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                };

                total += unbonded.amount;

                table.add_row(vec![
                    "Unbonded Stake".to_string(),
                    unbonded.try_format(&asset_cache).unwrap(),
                    format!("{:.4}", 1.0),
                    unbonded.try_format(&asset_cache).unwrap(),
                ]);

                let total = Value {
                    amount: total,
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                };

                table.add_row(vec![
                    "Total".to_string(),
                    total.try_format(&asset_cache).unwrap(),
                    String::new(),
                    String::new(),
                ]);
                println!("{}", table);
            }
            StakeCmd::ListValidators {
                show_inactive,
                detailed,
            } => {
                let mut client = app.oblivious_client().await?;

                let mut validators = client
                    .validator_info(ValidatorInfoRequest {
                        show_inactive: *show_inactive,
                        ..Default::default()
                    })
                    .await?
                    .into_inner()
                    .try_collect::<Vec<_>>()
                    .await?
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<validator::Info>, _>>()?;

                // Sort by voting power (descending), active first, then inactive
                validators.sort_by(|a, b| {
                    let av = if matches!(a.status.state, validator::State::Active) {
                        (a.status.voting_power, 0)
                    } else {
                        (0, a.status.voting_power)
                    };
                    let bv = if matches!(b.status.state, validator::State::Active) {
                        (b.status.voting_power, 0)
                    } else {
                        (0, b.status.voting_power)
                    };

                    bv.cmp(&av)
                });

                let total_voting_power = validators
                    .iter()
                    .filter_map(|v| {
                        if let validator::State::Active = v.status.state {
                            Some(v.status.voting_power)
                        } else {
                            None
                        }
                    })
                    .sum::<u64>() as f64;

                let mut table = Table::new();
                table.load_preset(presets::NOTHING);
                table.set_header(vec![
                    "Voting Power",
                    "Share",
                    "Commission",
                    "State",
                    "Bonding State",
                    "Validator Info",
                ]);

                for v in validators {
                    let voting_power = (v.status.voting_power as f64) * 1e-6; // apply udelegation factor
                    let active_voting_power = if matches!(v.status.state, validator::State::Active)
                    {
                        v.status.voting_power as f64
                    } else {
                        0.0
                    };
                    let power_percent = 100.0 * active_voting_power / total_voting_power;
                    let commission_bps = v
                        .validator
                        .funding_streams
                        .as_ref()
                        .iter()
                        .map(|fs| fs.rate_bps)
                        .sum::<u16>();

                    table.add_row(vec![
                        format!("{:.3}", voting_power),
                        format!("{:.2}%", power_percent),
                        format!("{}bps", commission_bps),
                        v.status.state.to_string(),
                        v.status.bonding_state.to_string(),
                        // TODO: consider rewriting this with term colors
                        // at some point, when we get around to it
                        format!("\x1b[1;31m{}\x1b[0m", v.validator.identity_key),
                    ]);
                    table.add_row(vec![
                        "".into(),
                        "".into(),
                        "".into(),
                        "".into(),
                        "".into(),
                        format!("  \x1b[1;92m{}\x1b[0m", v.validator.name),
                    ]);
                    if *detailed {
                        table.add_row(vec![
                            "".into(),
                            "".into(),
                            "".into(),
                            "".into(),
                            "".into(),
                            format!("  {}", v.validator.description),
                        ]);
                        table.add_row(vec![
                            "".into(),
                            "".into(),
                            "".into(),
                            "".into(),
                            "".into(),
                            format!("  {}", v.validator.website),
                        ]);
                    }
                }

                println!("{}", table);
            }
        }

        Ok(())
    }
}
