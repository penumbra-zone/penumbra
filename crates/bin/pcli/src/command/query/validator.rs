use std::{fs::File, io::Write, ops::RangeInclusive, time::Duration};

use anyhow::{Context, Result};
use colored::Colorize;
use comfy_table::{presets, Table};
use futures::TryStreamExt;
use penumbra_app::params::AppParameters;
use penumbra_num::Amount;
use penumbra_proto::core::app::v1::{
    query_service_client::QueryServiceClient as AppQueryServiceClient, AppParametersRequest,
};
use penumbra_proto::core::component::stake::v1::{
    query_service_client::QueryServiceClient as StakeQueryServiceClient, ValidatorInfoRequest,
    ValidatorStatusRequest, ValidatorUptimeRequest,
};
use penumbra_stake::{
    validator::{self, ValidatorToml},
    IdentityKey, Uptime,
};

use crate::App;

// TODO: replace this with something more standard for the `query` subcommand
#[derive(Debug, clap::Subcommand)]
pub enum ValidatorCmd {
    /// List all the validators in the network.
    List {
        /// Whether to show validators that are not currently part of the consensus set.
        #[clap(short = 'i', long)]
        show_inactive: bool,
        /// Whether to show detailed validator info.
        #[clap(short, long)]
        detailed: bool,
    },
    /// Fetch the current definition for a particular validator.
    Definition {
        /// The JSON file to write the definition to [default: stdout].
        #[clap(long)]
        file: Option<String>,
        /// The identity key of the validator to fetch.
        identity_key: String,
    },
    /// Get the uptime of the validator.
    Uptime {
        /// The identity key of the validator to fetch.
        identity_key: String,
    },
}

impl ValidatorCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            ValidatorCmd::List {
                show_inactive,
                detailed,
            } => {
                let mut client = StakeQueryServiceClient::new(app.pd_channel().await?);

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
                        (a.status.voting_power, Amount::zero())
                    } else {
                        (Amount::zero(), a.status.voting_power)
                    };
                    let bv = if matches!(b.status.state, validator::State::Active) {
                        (b.status.voting_power, Amount::zero())
                    } else {
                        (Amount::zero(), b.status.voting_power)
                    };

                    bv.cmp(&av)
                });

                let total_voting_power = validators
                    .iter()
                    .filter_map(|v| {
                        if let validator::State::Active = v.status.state {
                            Some(v.status.voting_power.value())
                        } else {
                            None
                        }
                    })
                    .sum::<u128>() as f64;

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
                    let voting_power = (v.status.voting_power.value() as f64) * 1e-6; // apply udelegation factor
                    let active_voting_power = if matches!(v.status.state, validator::State::Active)
                    {
                        v.status.voting_power.value() as f64
                    } else {
                        0.0
                    };
                    let power_percent = 100.0 * active_voting_power / total_voting_power;
                    let commission_bps = v
                        .validator
                        .funding_streams
                        .as_ref()
                        .iter()
                        .map(|fs| fs.rate_bps())
                        .sum::<u16>();

                    table.add_row(vec![
                        format!("{voting_power:.3}"),
                        format!("{power_percent:.2}%"),
                        format!("{commission_bps}bps"),
                        v.status.state.to_string(),
                        v.status.bonding_state.to_string(),
                        // TODO: consider rewriting this with term colors
                        // at some point, when we get around to it
                        v.validator.identity_key.to_string().red().to_string(),
                    ]);
                    table.add_row(vec![
                        "".into(),
                        "".into(),
                        "".into(),
                        "".into(),
                        "".into(),
                        v.validator.name.to_string().bright_green().to_string(),
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

                println!("{table}");
            }
            ValidatorCmd::Definition { file, identity_key } => {
                let identity_key = identity_key.parse::<IdentityKey>()?;

                /*
                use penumbra_proto::client::specific::ValidatorStatusRequest;

                let mut client = opt.specific_client().await?;
                let status: ValidatorStatus = client
                    .validator_status(ValidatorStatusRequest {
                        chain_id: "".to_string(), // TODO: fill in
                        identity_key: Some(identity_key.into()),
                    })
                    .await?
                    .into_inner()
                    .try_into()?;

                // why isn't the validator definition part of the status?
                // why do we have all these different validator messages?
                // do we need them?
                status.state.
                */

                // Intsead just download everything
                let mut client = StakeQueryServiceClient::new(app.pd_channel().await?);

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

                let validator: ValidatorToml = validators
                    .iter()
                    .map(|info| &info.validator)
                    .find(|v| v.identity_key == identity_key)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Could not find validator {}", identity_key))?
                    .into();

                if let Some(file) = file {
                    File::create(file)
                        .with_context(|| format!("cannot create file {file:?}"))?
                        .write_all(toml::to_string_pretty(&validator)?.as_bytes())
                        .context("could not write file")?;
                } else {
                    println!("{}", toml::to_string_pretty(&validator)?);
                }
            }
            ValidatorCmd::Uptime { identity_key } => {
                let identity_key = identity_key.parse::<IdentityKey>()?;

                let mut client = StakeQueryServiceClient::new(app.pd_channel().await?);

                // What's the uptime?
                let uptime: Uptime = client
                    .validator_uptime(ValidatorUptimeRequest {
                        identity_key: Some(identity_key.into()),
                    })
                    .await?
                    .into_inner()
                    .uptime
                    .ok_or_else(|| anyhow::anyhow!("uptime must be present in response"))?
                    .try_into()?;

                // Is the validator active?
                let status: validator::Status = client
                    .validator_status(ValidatorStatusRequest {
                        identity_key: Some(identity_key.into()),
                    })
                    .await?
                    .into_inner()
                    .status
                    .ok_or_else(|| anyhow::anyhow!("status must be present in response"))?
                    .try_into()?;
                let state = status.state;
                let active = matches!(state, validator::State::Active);

                // Get the chain parameters
                let mut client = AppQueryServiceClient::new(app.pd_channel().await?);
                let params: AppParameters = client
                    .app_parameters(tonic::Request::new(AppParametersRequest {}))
                    .await?
                    .into_inner()
                    .app_parameters
                    .ok_or_else(|| anyhow::anyhow!("empty AppParametersResponse message"))?
                    .try_into()?;

                let as_of_height = uptime.as_of_height();
                let missed_blocks = uptime.num_missed_blocks();
                let window_len = uptime.missed_blocks_window();

                let mut downtime_ranges: Vec<RangeInclusive<u64>> = Vec::new();
                for missed_block in uptime.missed_blocks() {
                    if let Some(range) = downtime_ranges.last_mut() {
                        if range.end() + 1 == missed_block {
                            *range = *range.start()..=missed_block;
                        } else {
                            downtime_ranges.push(missed_block..=missed_block);
                        }
                    } else {
                        downtime_ranges.push(missed_block..=missed_block);
                    }
                }

                let percent_uptime =
                    100.0 * (window_len as f64 - missed_blocks as f64) / window_len as f64;
                let min_uptime_blocks =
                    window_len as u64 - params.stake_params.missed_blocks_maximum;
                let percent_min_uptime = 100.0 * min_uptime_blocks as f64 / window_len as f64;
                let percent_downtime = 100.0 * missed_blocks as f64 / window_len as f64;
                let percent_downtime_penalty =
                    // Converting from basis points squared to percentage
                    100.0 * params.stake_params.slashing_penalty_downtime as f64 * 100.0;
                let min_remaining_downtime_blocks = (window_len as u64)
                    .saturating_sub(missed_blocks as u64)
                    .saturating_sub(min_uptime_blocks);
                let min_remaining_downtime = humantime::Duration::from(Duration::from_secs(
                    (min_remaining_downtime_blocks * 5) as u64,
                ));

                println!("Current uptime: {percent_uptime}% as of height {as_of_height}");
                let state_note = if active {
                    " (delegated funds are at stake)"
                } else {
                    ""
                };
                println!("Current status: {state}{state_note}");
                if active {
                    println!("Minimum uptime: {min_uptime_blocks}/{window_len} blocks ({percent_min_uptime}%)");
                }
                if !downtime_ranges.is_empty() {
                    let s = if missed_blocks == 1 { "" } else { "s" };
                    println!(
                        "Validator recently missed signing {missed_blocks}/{window_len} block{s} ({percent_downtime}%):"
                    );
                }
                for range in downtime_ranges.iter() {
                    let blocks = range.end() - range.start() + 1;
                    let s = if blocks == 1 { "" } else { "s" };
                    println!("  • {range:?} ({blocks} block{s})");
                }
                if active {
                    println!(
                        "Downtime penalty: {percent_downtime_penalty}% penalty will be applied if uptime falls below minimum"
                    );
                    if !downtime_ranges.is_empty() {
                        println!(
                            "Minimum remaining consecutive downtime before penalty: {min_remaining_downtime_blocks} blocks (approximately {min_remaining_downtime})"
                        );
                    }
                };
            }
        }

        Ok(())
    }
}
