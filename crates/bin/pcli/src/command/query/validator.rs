use std::{
    fs::File,
    io::Write,
    ops::{Deref, RangeInclusive},
    time::Duration,
};

use anyhow::{anyhow, Context, Error, Result};
use colored::Colorize;
use comfy_table::{presets, Table};
use futures::TryStreamExt;
use penumbra_sdk_app::params::AppParameters;
use penumbra_sdk_num::{fixpoint::U128x128, Amount};
use penumbra_sdk_proto::{
    core::{
        app::v1::{
            query_service_client::QueryServiceClient as AppQueryServiceClient, AppParametersRequest,
        },
        component::stake::v1::{
            query_service_client::QueryServiceClient as StakeQueryServiceClient,
            GetValidatorInfoRequest, GetValidatorInfoResponse, ValidatorInfoRequest,
            ValidatorStatusRequest, ValidatorUptimeRequest,
        },
    },
    DomainType,
};
use penumbra_sdk_stake::{
    rate::RateData,
    validator::{self, Info, Status, Validator, ValidatorToml},
    IdentityKey, Uptime, BPS_SQUARED_SCALING_FACTOR,
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
    /// Fetch the current status for a particular validator.
    Status {
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
                // Parse the identity key and construct the RPC request.
                let request = tonic::Request::new(GetValidatorInfoRequest {
                    identity_key: identity_key
                        .parse::<IdentityKey>()
                        .map(|ik| ik.to_proto())
                        .map(Some)?,
                });

                // Instantiate an RPC client and send the request.
                let GetValidatorInfoResponse { validator_info } = app
                    .pd_channel()
                    .await
                    .map(StakeQueryServiceClient::new)?
                    .get_validator_info(request)
                    .await?
                    .into_inner();

                // Coerce the validator information into TOML, or return an error if it was not
                // found within the client's response.
                let serialize = |v| toml::to_string_pretty(&v).map_err(Error::from);
                let toml = validator_info
                    .ok_or_else(|| anyhow!("response did not include validator info"))?
                    .try_into()
                    .context("parsing validator info")
                    .map(|Info { validator, .. }| validator)
                    .map(ValidatorToml::from)
                    .and_then(serialize)?;

                // Write to a file if an output file was specified, otherwise print to stdout.
                if let Some(file) = file {
                    File::create(file)
                        .with_context(|| format!("cannot create file {file:?}"))?
                        .write_all(toml.as_bytes())
                        .context("could not write file")?;
                } else {
                    println!("{}", toml);
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

                let mut downtime_ranges: Vec<RangeInclusive<u64>> = vec![];
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
                let signed_blocks = window_len as u64 - missed_blocks as u64;
                let min_uptime_blocks =
                    window_len as u64 - params.stake_params.missed_blocks_maximum;
                let percent_min_uptime = 100.0 * min_uptime_blocks as f64 / window_len as f64;
                let percent_max_downtime =
                    100.0 * params.stake_params.missed_blocks_maximum as f64 / window_len as f64;
                let percent_downtime = 100.0 * missed_blocks as f64 / window_len as f64;
                let percent_downtime_penalty =
                    // Converting from basis points squared to percentage: basis point ^ 2  %-age
                    //                                                    /--------------\/-----\
                    params.stake_params.slashing_penalty_downtime as f64 / 100.0 / 100.0 / 100.0;
                let min_remaining_downtime_blocks = (window_len as u64)
                    .saturating_sub(missed_blocks as u64)
                    .saturating_sub(min_uptime_blocks);
                let min_remaining_downtime = humantime::Duration::from(Duration::from_secs(
                    (min_remaining_downtime_blocks * 5) as u64,
                ));
                let cumulative_downtime =
                    humantime::Duration::from(Duration::from_secs((missed_blocks * 5) as u64));
                let percent_grace = 100.0 * min_remaining_downtime_blocks as f64
                    / (window_len - min_uptime_blocks as usize) as f64;
                let window_len_len = window_len.to_string().len();

                println!("{state} validator: as of block {as_of_height}");
                println!("Achieved signing: {percent_uptime:>6.2}% = {signed_blocks:width$}/{window_len} most-recent blocks", width = window_len_len);
                if active {
                    println!("Required signing: {percent_min_uptime:>6.2}% = {min_uptime_blocks:width$}/{window_len} most-recent blocks", width = window_len_len);
                }
                println!("Salient downtime: {percent_downtime:>6.2}% = {missed_blocks:width$}/{window_len} most-recent blocks ~ {cumulative_downtime} cumulative downtime", width = window_len_len);
                if active {
                    println!("Unexpended grace: {percent_grace:>6.2}% = {min_remaining_downtime_blocks:width$}/{window_len} forthcoming blocks ~ {min_remaining_downtime} at minimum before penalty", width = window_len_len);
                    println!( "Downtime penalty: {percent_downtime_penalty:>6.2}% - if downtime exceeds {percent_max_downtime:.2}%, penalty will be applied to all delegations");
                }
                if !downtime_ranges.is_empty() {
                    println!("Downtime details:");
                    let mut max_blocks_width = 0;
                    let mut max_start_width = 0;
                    let mut max_end_width = 0;
                    for range in downtime_ranges.iter() {
                        let blocks = range.end() - range.start() + 1;
                        max_blocks_width = max_blocks_width.max(blocks.to_string().len());
                        max_start_width = max_start_width.max(range.start().to_string().len());
                        if blocks != 1 {
                            max_end_width = max_end_width.max(range.end().to_string().len());
                        }
                    }
                    for range in downtime_ranges.iter() {
                        let blocks = range.end() - range.start() + 1;
                        let estimated_duration =
                            humantime::Duration::from(Duration::from_secs((blocks * 5) as u64));
                        if blocks == 1 {
                            let height = range.start();
                            println!(
                                "  • {blocks:width$} missed:  block {height:>height_width$} {empty:>duration_width$}(~ {estimated_duration})",
                                width = max_blocks_width,
                                height_width = max_start_width,
                                duration_width = max_end_width + 5,
                                empty = "",
                            );
                        } else {
                            let start = range.start();
                            let end = range.end();
                            println!(
                                "  • {blocks:width$} missed: blocks {start:>start_width$} ..= {end:>end_width$} (~ {estimated_duration})",
                                width = max_blocks_width,
                                start_width = max_start_width,
                                end_width = max_end_width,
                            );
                        };
                    }
                }
            }
            ValidatorCmd::Status { identity_key } => {
                // Parse the identity key and construct the RPC request.
                let request = tonic::Request::new(GetValidatorInfoRequest {
                    identity_key: identity_key
                        .parse::<IdentityKey>()
                        .map(|ik| ik.to_proto())
                        .map(Some)?,
                });

                // Instantiate an RPC client and send the request.
                let GetValidatorInfoResponse { validator_info } = app
                    .pd_channel()
                    .await
                    .map(StakeQueryServiceClient::new)?
                    .get_validator_info(request)
                    .await?
                    .into_inner();

                // Parse the validator status, or return an error if it was not found within the
                // client's response.
                let info = validator_info
                    .ok_or_else(|| anyhow!("response did not include validator info"))?
                    .try_into()
                    .context("parsing validator info")?;

                // Initialize a table, add a header and insert this validator's information.
                let mut table = Table::new();
                table
                    .load_preset(presets::NOTHING)
                    .set_header(vec![
                        "Voting Power",
                        "Commission",
                        "State",
                        "Bonding State",
                        "Exchange Rate",
                        "Identity Key",
                        "Name",
                    ])
                    .add_row(StatusRow::new(info));
                println!("{table}");
            }
        }

        Ok(())
    }
}

/// A row within the `status` command's table output.
struct StatusRow {
    power: f64,
    commission: u16,
    state: validator::State,
    bonding_state: validator::BondingState,
    exchange_rate: U128x128,
    identity_key: IdentityKey,
    name: String,
}

impl StatusRow {
    /// Constructs a new [`StatusRow`].
    fn new(
        Info {
            validator:
                Validator {
                    funding_streams,
                    identity_key,
                    name,
                    ..
                },
            status:
                Status {
                    state,
                    bonding_state,
                    voting_power,
                    ..
                },
            rate_data:
                RateData {
                    validator_exchange_rate,
                    ..
                },
        }: Info,
    ) -> Self {
        // Calculate the scaled voting power, exchange rate, and commissions.
        let power = (voting_power.value() as f64) * 1e-6;
        let commission = funding_streams.iter().map(|fs| fs.rate_bps()).sum();
        let exchange_rate = {
            let rate_bps_sq = U128x128::from(validator_exchange_rate);
            (rate_bps_sq / BPS_SQUARED_SCALING_FACTOR.deref()).expect("nonzero scaling factor")
        };

        Self {
            power,
            commission,
            state,
            bonding_state,
            exchange_rate,
            identity_key,
            name,
        }
    }
}

impl Into<comfy_table::Row> for StatusRow {
    fn into(self) -> comfy_table::Row {
        let Self {
            power,
            commission,
            state,
            bonding_state,
            exchange_rate,
            identity_key,
            name,
        } = self;

        [
            format!("{power:.3}"),
            format!("{commission}bps"),
            state.to_string(),
            bonding_state.to_string(),
            exchange_rate.to_string(),
            identity_key.to_string(),
            name,
        ]
        .into()
    }
}
