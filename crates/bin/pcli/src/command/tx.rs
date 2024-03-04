use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use decaf377::{Fq, Fr};
use ibc_proto::ibc::core::client::v1::{
    query_client::QueryClient as IbcClientQueryClient, QueryClientStateRequest,
};
use ibc_proto::ibc::core::connection::v1::query_client::QueryClient as IbcConnectionQueryClient;
use ibc_proto::ibc::core::{
    channel::v1::{query_client::QueryClient as IbcChannelQueryClient, QueryChannelRequest},
    connection::v1::QueryConnectionRequest,
};
use ibc_types::core::{
    channel::{ChannelId, PortId},
    client::Height as IbcHeight,
};
use ibc_types::lightclients::tendermint::client_state::ClientState as TendermintClientState;
use rand_core::OsRng;
use regex::Regex;

use liquidity_position::PositionCmd;
use penumbra_asset::{asset, asset::Metadata, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_dex::{lp::position, swap_claim::SwapClaimPlan};
use penumbra_fee::Fee;
use penumbra_governance::{proposal::ProposalToml, proposal_state::State as ProposalState, Vote};
use penumbra_keys::keys::AddressIndex;
use penumbra_num::Amount;
use penumbra_proto::{
    core::component::{
        dex::v1::{
            query_service_client::QueryServiceClient as DexQueryServiceClient,
            LiquidityPositionByIdRequest, PositionId,
        },
        governance::v1::{
            query_service_client::QueryServiceClient as GovernanceQueryServiceClient,
            NextProposalIdRequest, ProposalDataRequest, ProposalInfoRequest, ProposalInfoResponse,
            ProposalRateDataRequest,
        },
        sct::v1::{
            query_service_client::QueryServiceClient as SctQueryServiceClient, EpochByHeightRequest,
        },
        stake::v1::{
            query_service_client::QueryServiceClient as StakeQueryServiceClient,
            ValidatorPenaltyRequest,
        },
    },
    view::v1::GasPricesRequest,
};
use penumbra_shielded_pool::Ics20Withdrawal;
use penumbra_stake::rate::RateData;
use penumbra_stake::{DelegationToken, IdentityKey, Penalty, UnbondingToken, UndelegateClaimPlan};
use penumbra_transaction::{gas::swap_claim_gas_cost, memo::MemoPlaintext};
use penumbra_view::ViewClient;
use penumbra_wallet::plan::{self, Planner};
use proposal::ProposalCmd;

use crate::App;

mod liquidity_position;
mod proposal;
mod replicate;

#[derive(Debug, clap::Subcommand)]
pub enum TxCmd {
    /// Send funds to a Penumbra address.
    #[clap(display_order = 100)]
    Send {
        /// The destination address to send funds to.
        #[clap(long, display_order = 100)]
        to: String,
        /// The amounts to send, written as typed values 1.87penumbra, 12cubes, etc.
        values: Vec<String>,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
        /// Optional. Set the transaction's memo field to the provided text.
        #[clap(long)]
        memo: Option<String>,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Deposit stake into a validator's delegation pool.
    #[clap(display_order = 200)]
    Delegate {
        /// The identity key of the validator to delegate to.
        #[clap(long, display_order = 100)]
        to: String,
        /// The amount of stake to delegate.
        amount: String,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Withdraw stake from a validator's delegation pool.
    #[clap(display_order = 200)]
    Undelegate {
        /// The amount of delegation tokens to undelegate.
        amount: String,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Claim any undelegations that have finished unbonding.
    #[clap(display_order = 200)]
    UndelegateClaim {
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Swap tokens of one denomination for another using the DEX.
    ///
    /// Swaps are batched and executed at the market-clearing price.
    ///
    /// A swap generates two transactions: an initial "swap" transaction that
    /// submits the swap, and a "swap claim" transaction that privately mints
    /// the output funds once the batch has executed.  The second transaction
    /// will be created and submitted automatically.
    #[clap(display_order = 300)]
    Swap {
        /// The input amount to swap, written as a typed value 1.87penumbra, 12cubes, etc.
        input: String,
        /// The denomination to swap the input into, e.g. `gm`
        #[clap(long, display_order = 100)]
        into: String,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Vote on a governance proposal in your role as a delegator (see also: `pcli validator vote`).
    #[clap(display_order = 400)]
    Vote {
        /// Only spend funds and vote with staked delegation tokens originally received by the given
        /// account.
        #[clap(long, default_value = "0", global = true, display_order = 300)]
        source: u32,
        #[clap(subcommand)]
        vote: VoteCmd,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Submit or withdraw a governance proposal.
    #[clap(display_order = 500, subcommand)]
    Proposal(ProposalCmd),
    /// Deposit funds into the Community Pool.
    #[clap(display_order = 600)]
    CommunityPoolDeposit {
        /// The amounts to send, written as typed values 1.87penumbra, 12cubes, etc.
        values: Vec<String>,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Manage liquidity positions.
    #[clap(display_order = 500, subcommand, visible_alias = "lp")]
    Position(PositionCmd),
    /// Consolidate many small notes into a few larger notes.
    ///
    /// Since Penumbra transactions reveal their arity (how many spends,
    /// outputs, etc), but transactions are unlinkable from each other, it is
    /// slightly preferable to sweep small notes into larger ones in an isolated
    /// "sweep" transaction, rather than at the point that they should be spent.
    ///
    /// Currently, only zero-fee sweep transactions are implemented.
    #[clap(display_order = 990)]
    Sweep,

    /// Perform an ICS-20 withdrawal, moving funds from the Penumbra chain
    /// to a counterparty chain.
    ///
    /// For a withdrawal to be processed on the counterparty, IBC packets must be relayed between
    /// the two chains. Relaying is out of scope for the `pcli` tool.
    #[clap(display_order = 250)]
    Withdraw {
        /// Address on the receiving chain,
        /// e.g. cosmos1grgelyng2v6v3t8z87wu3sxgt9m5s03xvslewd. The chain_id for the counterparty
        /// chain will be discovered automatically, based on the `--channel` setting.
        #[clap(long)]
        to: String,

        /// The value to withdraw, eg "1000upenumbra"
        value: String,

        /// The IBC channel on the primary Penumbra chain to use for performing the withdrawal.
        /// This channel must already exist, as configured by a relayer client.
        /// You can search for channels via e.g. `pcli query ibc channel transfer 0`.
        #[clap(long)]
        channel: u64,

        /// Block height on the counterparty chain, after which the withdrawal will be considered
        /// invalid if not already relayed. Must be specified as a tuple of revision number and block
        /// height, e.g. `5-1000000` means "chain revision 5, block height of 1000000".
        /// You must know the chain id of the counterparty chain beforehand, e.g. `osmosis-testnet-5`,
        /// to know the revision number.
        #[clap(long, display_order = 100)]
        timeout_height: Option<IbcHeight>,
        /// Timestamp, specified in epoch time, after which the withdrawal will be considered
        /// invalid if not already relayed.
        #[clap(long, default_value = "0", display_order = 150)]
        timeout_timestamp: u64,

        /// Only withdraw funds from the specified wallet id within Penumbra.
        #[clap(long, default_value = "0", display_order = 200)]
        source: u32,

        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
    },
}

// A fee tier enum suitable for use with clap.
#[derive(Copy, Clone, clap::ValueEnum, Debug)]
pub enum FeeTier {
    Low,
    Medium,
    High,
}

impl Default for FeeTier {
    fn default() -> Self {
        Self::Low
    }
}

// Convert from the internal fee tier enum to the clap-compatible enum.
impl From<penumbra_fee::FeeTier> for FeeTier {
    fn from(tier: penumbra_fee::FeeTier) -> Self {
        match tier {
            penumbra_fee::FeeTier::Low => Self::Low,
            penumbra_fee::FeeTier::Medium => Self::Medium,
            penumbra_fee::FeeTier::High => Self::High,
        }
    }
}

// Convert from the the clap-compatible fee tier enum to the internal fee tier enum.
impl From<FeeTier> for penumbra_fee::FeeTier {
    fn from(tier: FeeTier) -> Self {
        match tier {
            FeeTier::Low => Self::Low,
            FeeTier::Medium => Self::Medium,
            FeeTier::High => Self::High,
        }
    }
}

/// Vote on a governance proposal.
#[derive(Debug, Clone, Copy, clap::Subcommand)]
pub enum VoteCmd {
    /// Vote in favor of a proposal.
    #[clap(display_order = 100)]
    Yes {
        /// The proposal ID to vote on.
        #[clap(long = "on")]
        proposal_id: u64,
    },
    /// Vote against a proposal.
    #[clap(display_order = 200)]
    No {
        /// The proposal ID to vote on.
        #[clap(long = "on")]
        proposal_id: u64,
    },
    /// Abstain from voting on a proposal.
    #[clap(display_order = 300)]
    Abstain {
        /// The proposal ID to vote on.
        #[clap(long = "on")]
        proposal_id: u64,
    },
}

impl From<VoteCmd> for (u64, Vote) {
    fn from(cmd: VoteCmd) -> (u64, Vote) {
        match cmd {
            VoteCmd::Yes { proposal_id } => (proposal_id, Vote::Yes),
            VoteCmd::No { proposal_id } => (proposal_id, Vote::No),
            VoteCmd::Abstain { proposal_id } => (proposal_id, Vote::Abstain),
        }
    }
}

impl TxCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn offline(&self) -> bool {
        match self {
            TxCmd::Send { .. } => false,
            TxCmd::Sweep { .. } => false,
            TxCmd::Swap { .. } => false,
            TxCmd::Delegate { .. } => false,
            TxCmd::Undelegate { .. } => false,
            TxCmd::UndelegateClaim { .. } => false,
            TxCmd::Vote { .. } => false,
            TxCmd::Proposal(proposal_cmd) => proposal_cmd.offline(),
            TxCmd::CommunityPoolDeposit { .. } => false,
            TxCmd::Position(lp_cmd) => lp_cmd.offline(),
            TxCmd::Withdraw { .. } => false,
        }
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let gas_prices = app
            .view
            .as_mut()
            .context("view service must be initialized")?
            .gas_prices(GasPricesRequest {})
            .await?
            .into_inner()
            .gas_prices
            .expect("gas prices must be available")
            .try_into()?;

        match self {
            TxCmd::Send {
                values,
                to,
                source: from,
                memo,
                fee_tier,
            } => {
                // Parse all of the values provided.
                let values = values
                    .iter()
                    .map(|v| v.parse())
                    .collect::<Result<Vec<Value>, _>>()?;
                let to = to
                    .parse()
                    .map_err(|_| anyhow::anyhow!("address is invalid"))?;

                let return_address = app
                    .config
                    .full_viewing_key
                    .payment_address((*from).into())
                    .0;

                let memo_plaintext =
                    MemoPlaintext::new(return_address, memo.clone().unwrap_or_default())?;

                let mut planner = Planner::new(OsRng);

                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());
                for value in values.iter().cloned() {
                    planner.output(value, to);
                }
                let plan = planner
                    .memo(memo_plaintext)?
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*from),
                    )
                    .await
                    .context("can't build send transaction")?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::CommunityPoolDeposit {
                values,
                source,
                fee_tier,
            } => {
                let values = values
                    .iter()
                    .map(|v| v.parse())
                    .collect::<Result<Vec<Value>, _>>()?;

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());
                for value in values {
                    planner.community_pool_deposit(value);
                }
                let plan = planner
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Sweep => loop {
                let plans = plan::sweep(
                    app.view
                        .as_mut()
                        .context("view service must be initialized")?,
                    OsRng,
                )
                .await?;
                let num_plans = plans.len();

                for (i, plan) in plans.into_iter().enumerate() {
                    println!("building sweep {i} of {num_plans}");
                    app.build_and_submit_transaction(plan).await?;
                }
                if num_plans == 0 {
                    println!("finished sweeping");
                    break;
                }
            },
            TxCmd::Swap {
                input,
                into,
                source,
                fee_tier,
            } => {
                let input = input.parse::<Value>()?;
                let into = asset::REGISTRY.parse_unit(into.as_str()).base();

                let fvk = app.config.full_viewing_key.clone();

                // If a source address was specified, use it for the swap, otherwise,
                // use the default address.
                let (claim_address, _dtk_d) =
                    fvk.incoming().payment_address(AddressIndex::new(*source));

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices.clone())
                    .set_fee_tier((*fee_tier).into());
                // The swap claim requires a pre-paid fee, however gas costs might change in the meantime.
                // This shouldn't be an issue, since the planner will account for the difference and add additional
                // spends alongside the swap claim transaction as necessary.
                //
                // Regardless, we apply a gas adjustment factor of 2.0 up-front to reduce the likelihood of
                // requiring an additional spend at the time of claim.
                //
                // Since the swap claim fee needs to be passed in to the planner to build the swap (it is
                // part of the `SwapPlaintext`), we can't use the planner to estimate the fee and need to
                // call the helper method directly.
                let estimated_claim_fee = Fee::from_staking_token_amount(
                    Amount::from(2u32) * gas_prices.fee(&swap_claim_gas_cost()),
                );
                planner.swap(input, into.id(), estimated_claim_fee, claim_address)?;

                let plan = planner
                    .plan(app.view(), AddressIndex::new(*source))
                    .await
                    .context("can't plan swap transaction")?;

                // Hold on to the swap plaintext to be able to claim.
                let swap_plaintext = plan
                    .swap_plans()
                    .next()
                    .expect("swap plan must be present")
                    .swap_plaintext
                    .clone();

                // Submit the `Swap` transaction, waiting for confirmation,
                // at which point the swap will be available for claiming.
                app.build_and_submit_transaction(plan).await?;

                // Fetch the SwapRecord with the claimable swap.
                let swap_record = app
                    .view()
                    .swap_by_commitment(swap_plaintext.swap_commitment())
                    .await?;

                let asset_cache = app.view().assets().await?;

                let pro_rata_outputs = swap_record
                    .output_data
                    .pro_rata_outputs((swap_plaintext.delta_1_i, swap_plaintext.delta_2_i));
                println!("Swap submitted and batch confirmed!");
                println!(
                    "You will receive outputs of {} and {}. Claiming now...",
                    Value {
                        amount: pro_rata_outputs.0,
                        asset_id: swap_record.output_data.trading_pair.asset_1()
                    }
                    .format(&asset_cache),
                    Value {
                        amount: pro_rata_outputs.1,
                        asset_id: swap_record.output_data.trading_pair.asset_2()
                    }
                    .format(&asset_cache),
                );

                let params = app
                    .view
                    .as_mut()
                    .context("view service must be initialized")?
                    .app_params()
                    .await?;

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());
                let plan = planner
                    .swap_claim(SwapClaimPlan {
                        swap_plaintext,
                        position: swap_record.position,
                        output_data: swap_record.output_data,
                        epoch_duration: params.sct_params.epoch_duration,
                        proof_blinding_r: Fq::rand(&mut OsRng),
                        proof_blinding_s: Fq::rand(&mut OsRng),
                    })
                    .plan(app.view(), AddressIndex::new(*source))
                    .await
                    .context("can't plan swap claim")?;

                // Submit the `SwapClaim` transaction.
                // BUG: this doesn't wait for confirmation, see
                // https://github.com/penumbra-zone/penumbra/pull/2091/commits/128b24a6303c2f855a708e35f9342987f1dd34ec
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Delegate {
                to,
                amount,
                source,
                fee_tier,
            } => {
                let unbonded_amount = {
                    let Value { amount, asset_id } = amount.parse::<Value>()?;
                    if asset_id != *STAKING_TOKEN_ASSET_ID {
                        anyhow::bail!("staking can only be done with the staking token");
                    }
                    amount
                };

                let to = to.parse::<IdentityKey>()?;

                let mut client = StakeQueryServiceClient::new(app.pd_channel().await?);
                let rate_data: RateData = client
                    .current_validator_rate(tonic::Request::new(to.into()))
                    .await?
                    .into_inner()
                    .try_into()?;

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());
                let plan = planner
                    .delegate(unbonded_amount, rate_data)
                    .plan(app.view(), AddressIndex::new(*source))
                    .await
                    .context("can't plan delegation")?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Undelegate {
                amount,
                source,
                fee_tier,
            } => {
                let delegation_value @ Value {
                    amount: _,
                    asset_id,
                } = amount.parse::<Value>()?;

                // TODO: it's awkward that we can't just pull the denom out of the `amount` string we were already given
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

                let mut client = StakeQueryServiceClient::new(app.pd_channel().await?);
                let rate_data: RateData = client
                    .current_validator_rate(tonic::Request::new(from.into()))
                    .await?
                    .into_inner()
                    .try_into()?;

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                let plan = planner
                    .undelegate(delegation_value.amount, rate_data)
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await
                    .context("can't build undelegate plan")?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::UndelegateClaim { fee_tier } => {
                let channel = app.pd_channel().await?;
                let view: &mut dyn ViewClient = app
                    .view
                    .as_mut()
                    .context("view service must be initialized")?;

                let current_height = view.status().await?.full_sync_height;
                let mut client = SctQueryServiceClient::new(channel.clone());
                let current_epoch = client
                    .epoch_by_height(EpochByHeightRequest {
                        height: current_height,
                    })
                    .await?
                    .into_inner()
                    .epoch
                    .context("unable to get epoch for current height")?;
                let asset_cache = view.assets().await?;

                // Query the view client for the list of undelegations that are ready to be claimed.
                // We want to claim them into the same address index that currently holds the tokens.
                let notes = view.unspent_notes_by_address_and_asset().await?;

                for (address_index, notes_by_asset) in notes.into_iter() {
                    for (token, notes) in
                        notes_by_asset.into_iter().filter_map(|(asset_id, notes)| {
                            // Filter for notes that are unbonding tokens.
                            let denom = asset_cache
                                .get(&asset_id)
                                .expect("asset ID should exist in asset cache")
                                .clone();
                            match UnbondingToken::try_from(denom) {
                                Ok(token) => Some((token, notes)),
                                Err(_) => None,
                            }
                        })
                    {
                        println!("claiming {}", token.denom().default_unit());
                        let validator_identity = token.validator();
                        let start_epoch_index = token.start_epoch_index();
                        let end_epoch_index = current_epoch.index;

                        let mut client = StakeQueryServiceClient::new(channel.clone());
                        let penalty: Penalty = client
                            .validator_penalty(tonic::Request::new(ValidatorPenaltyRequest {
                                identity_key: Some(validator_identity.into()),
                                start_epoch_index,
                                end_epoch_index,
                            }))
                            .await?
                            .into_inner()
                            .penalty
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "no penalty returned for validator {}",
                                    validator_identity
                                )
                            })?
                            .try_into()?;

                        let mut planner = Planner::new(OsRng);
                        planner
                            .set_gas_prices(gas_prices.clone())
                            .set_fee_tier((*fee_tier).into());
                        let unbonding_amount = notes.iter().map(|n| n.note.amount()).sum();
                        for note in notes {
                            planner.spend(note.note, note.position);
                        }

                        let plan = planner
                            .undelegate_claim(UndelegateClaimPlan {
                                validator_identity,
                                start_epoch_index,
                                penalty,
                                unbonding_amount,
                                balance_blinding: Fr::rand(&mut OsRng),
                                proof_blinding_r: Fq::rand(&mut OsRng),
                                proof_blinding_s: Fq::rand(&mut OsRng),
                            })
                            .plan(
                                app.view
                                    .as_mut()
                                    .context("view service must be initialized")?,
                                address_index,
                            )
                            .await?;
                        app.build_and_submit_transaction(plan).await?;
                    }
                }
            }
            TxCmd::Proposal(ProposalCmd::Submit {
                file,
                source,
                deposit_amount,
                fee_tier,
            }) => {
                let mut proposal_file = File::open(file).context("can't open proposal file")?;
                let mut proposal_string = String::new();
                proposal_file
                    .read_to_string(&mut proposal_string)
                    .context("can't read proposal file")?;
                let proposal_toml: ProposalToml =
                    toml::from_str(&proposal_string).context("can't parse proposal file")?;
                let proposal = proposal_toml
                    .try_into()
                    .context("can't parse proposal file")?;

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());
                let plan = planner
                    .proposal_submit(proposal, Amount::from(*deposit_amount))
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Proposal(ProposalCmd::Withdraw {
                proposal_id,
                reason,
                source,
                fee_tier,
            }) => {
                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());
                let plan = planner
                    .proposal_withdraw(*proposal_id, reason.clone())
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Proposal(ProposalCmd::Template { file, kind }) => {
                let app_params = app.view().app_params().await?;

                // Find out what the latest proposal ID is so we can include the next ID in the template:
                let mut client = GovernanceQueryServiceClient::new(app.pd_channel().await?);
                let next_proposal_id: u64 = client
                    .next_proposal_id(NextProposalIdRequest {})
                    .await?
                    .into_inner()
                    .next_proposal_id;

                let toml_template: ProposalToml = kind
                    .template_proposal(&app_params, next_proposal_id)?
                    .into();

                if let Some(file) = file {
                    File::create(file)
                        .with_context(|| format!("cannot create file {file:?}"))?
                        .write_all(toml::to_string_pretty(&toml_template)?.as_bytes())
                        .context("could not write file")?;
                } else {
                    println!("{}", toml::to_string_pretty(&toml_template)?);
                }
            }
            TxCmd::Proposal(ProposalCmd::DepositClaim {
                proposal_id,
                source,
                fee_tier,
            }) => {
                let mut client = GovernanceQueryServiceClient::new(app.pd_channel().await?);
                let proposal = client
                    .proposal_data(ProposalDataRequest {
                        proposal_id: *proposal_id,
                    })
                    .await?
                    .into_inner();
                let state: ProposalState = proposal
                    .state
                    .context(format!(
                        "proposal state for proposal {} was not found",
                        proposal_id
                    ))?
                    .try_into()?;
                let deposit_amount: Amount = proposal
                    .proposal_deposit_amount
                    .context(format!(
                        "proposal deposit amount for proposal {} was not found",
                        proposal_id
                    ))?
                    .try_into()?;

                let outcome = match state {
                    ProposalState::Voting => anyhow::bail!(
                        "proposal {} is still voting, so the deposit cannot yet be claimed",
                        proposal_id
                    ),
                    ProposalState::Withdrawn { reason: _ } => {
                        anyhow::bail!("proposal {} has been withdrawn but voting has not yet concluded, so the deposit cannot yet be claimed", proposal_id);
                    }
                    ProposalState::Finished { outcome } => outcome.map(|_| ()),
                    ProposalState::Claimed { outcome: _ } => {
                        anyhow::bail!("proposal {} has already been claimed", proposal_id)
                    }
                };

                let plan = Planner::new(OsRng)
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .proposal_deposit_claim(*proposal_id, deposit_amount, outcome)
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Vote {
                vote,
                source,
                fee_tier,
            } => {
                let (proposal_id, vote): (u64, Vote) = (*vote).into();

                // Before we vote on the proposal, we have to gather some information about it so
                // that we can prepare our vote:
                // - the start height, so we can select the votable staked notes to vote with
                // - the start position, so we can submit the appropriate public `start_position`
                //   input for stateless proof verification
                // - the rate data for every validator at the start of the proposal, so we can
                //   convert staked notes into voting power and mint the correct amount of voting
                //   receipt tokens to ourselves

                let mut client = GovernanceQueryServiceClient::new(app.pd_channel().await?);
                let ProposalInfoResponse {
                    start_block_height,
                    start_position,
                } = client
                    .proposal_info(ProposalInfoRequest { proposal_id })
                    .await?
                    .into_inner();
                let start_position = start_position.into();

                let mut rate_data_stream = client
                    .proposal_rate_data(ProposalRateDataRequest { proposal_id })
                    .await?
                    .into_inner();

                let mut start_rate_data = BTreeMap::new();
                while let Some(response) = rate_data_stream.message().await? {
                    let rate_data: RateData = response
                        .rate_data
                        .ok_or_else(|| {
                            anyhow::anyhow!("proposal rate data stream response missing rate data")
                        })?
                        .try_into()
                        .context("invalid rate data")?;
                    start_rate_data.insert(rate_data.identity_key.clone(), rate_data);
                }

                let plan = Planner::new(OsRng)
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .delegator_vote(
                        proposal_id,
                        start_block_height,
                        start_position,
                        start_rate_data,
                        vote,
                    )
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Position(PositionCmd::Order(order)) => {
                let asset_cache = app.view().assets().await?;

                tracing::info!(?order);
                let source = AddressIndex::new(order.source());
                let position = order.as_position(&asset_cache, OsRng)?;
                tracing::info!(?position);

                let plan = Planner::new(OsRng)
                    .set_gas_prices(gas_prices)
                    .set_fee_tier(order.fee_tier().into())
                    .position_open(position)
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        source,
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Withdraw {
                to,
                value,
                timeout_height,
                timeout_timestamp,
                channel,
                source,
                fee_tier,
            } => {
                let destination_chain_address = to;

                let (ephemeral_return_address, _) = app
                    .config
                    .full_viewing_key
                    .ephemeral_address(OsRng, AddressIndex::from(*source));

                let timeout_height = match timeout_height {
                    Some(h) => h.clone(),
                    None => {
                        // look up the height for the counterparty and add 2 days of block time
                        // (assuming 10 seconds per block) to it

                        // look up the client state from the channel by looking up channel id -> connection id -> client state
                        let mut ibc_channel_client =
                            IbcChannelQueryClient::new(app.pd_channel().await?);

                        let req = QueryChannelRequest {
                            port_id: PortId::transfer().to_string(),
                            channel_id: format!("channel-{}", channel),
                        };

                        let channel = ibc_channel_client
                            .channel(req)
                            .await?
                            .into_inner()
                            .channel
                            .ok_or_else(|| anyhow::anyhow!("channel not found"))?;

                        let connection_id = channel.connection_hops[0].clone();

                        let mut ibc_connection_client =
                            IbcConnectionQueryClient::new(app.pd_channel().await?);

                        let req = QueryConnectionRequest {
                            connection_id: connection_id.clone(),
                        };
                        let connection = ibc_connection_client
                            .connection(req)
                            .await?
                            .into_inner()
                            .connection
                            .ok_or_else(|| anyhow::anyhow!("connection not found"))?;

                        let mut ibc_client_client =
                            IbcClientQueryClient::new(app.pd_channel().await?);
                        let req = QueryClientStateRequest {
                            client_id: connection.client_id,
                        };
                        let client_state = ibc_client_client
                            .client_state(req)
                            .await?
                            .into_inner()
                            .client_state
                            .ok_or_else(|| anyhow::anyhow!("client state not found"))?;

                        let tm_client_state = TendermintClientState::try_from(client_state)?;

                        let last_update_height = tm_client_state.latest_height;

                        // 10 seconds per block, 2 days
                        let timeout_n_blocks = ((24 * 60 * 60) / 10) * 2;

                        IbcHeight {
                            revision_number: last_update_height.revision_number,
                            revision_height: last_update_height.revision_height + timeout_n_blocks,
                        }
                    }
                };

                // get the current time on the local machine
                let current_time_u64_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_nanos() as u64;

                let mut timeout_timestamp = *timeout_timestamp;
                if timeout_timestamp == 0u64 {
                    // add 2 days to current time
                    timeout_timestamp = current_time_u64_ms + 1.728e14 as u64;
                }

                fn parse_denom_and_amount(value_str: &str) -> anyhow::Result<(Amount, Metadata)> {
                    let denom_re = Regex::new(r"^([0-9.]+)(.+)$").context("denom regex invalid")?;
                    if let Some(captures) = denom_re.captures(value_str) {
                        let numeric_str = captures.get(1).expect("matched regex").as_str();
                        let denom_str = captures.get(2).expect("matched regex").as_str();

                        let display_denom = asset::REGISTRY.parse_unit(denom_str);
                        let amount = display_denom.parse_value(numeric_str)?;
                        let denom = display_denom.base();

                        Ok((amount, denom))
                    } else {
                        Err(anyhow::anyhow!("could not parse value"))
                    }
                }

                let (amount, denom) = parse_denom_and_amount(value)?;

                let withdrawal = Ics20Withdrawal {
                    destination_chain_address: destination_chain_address.to_string(),
                    denom,
                    amount,
                    timeout_height,
                    timeout_time: timeout_timestamp,
                    return_address: ephemeral_return_address,
                    // TODO: impl From<u64> for ChannelId
                    source_channel: ChannelId::from_str(format!("channel-{}", channel).as_ref())?,
                };

                let plan = Planner::new(OsRng)
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .ics20_withdrawal(withdrawal)
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Position(PositionCmd::Close {
                position_id,
                source,
                fee_tier,
            }) => {
                let plan = Planner::new(OsRng)
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .position_close(*position_id)
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Position(PositionCmd::CloseAll {
                source,
                trading_pair,
                fee_tier,
            }) => {
                let view: &mut dyn ViewClient = app
                    .view
                    .as_mut()
                    .context("view service must be initialized")?;

                let owned_position_ids = view
                    .owned_position_ids(Some(position::State::Opened), *trading_pair)
                    .await?;

                if owned_position_ids.is_empty() {
                    println!("No open positions are available to close.");
                    return Ok(());
                }

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                for position_id in owned_position_ids {
                    // Close the position
                    planner.position_close(position_id);
                }

                let final_plan = planner
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(final_plan).await?;
            }
            TxCmd::Position(PositionCmd::WithdrawAll {
                source,
                trading_pair,
                fee_tier,
            }) => {
                let view: &mut dyn ViewClient = app
                    .view
                    .as_mut()
                    .context("view service must be initialized")?;

                let owned_position_ids = view
                    .owned_position_ids(Some(position::State::Closed), *trading_pair)
                    .await?;

                if owned_position_ids.is_empty() {
                    println!("No closed positions are available to withdraw.");
                    return Ok(());
                }

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                let mut client = DexQueryServiceClient::new(app.pd_channel().await?);

                for position_id in owned_position_ids {
                    // Withdraw the position

                    // Fetch the information regarding the position from the view service.
                    let position = client
                        .liquidity_position_by_id(LiquidityPositionByIdRequest {
                            position_id: Some(position_id.into()),
                        })
                        .await?
                        .into_inner();

                    let reserves = position
                        .data
                        .clone()
                        .expect("missing position metadata")
                        .reserves
                        .expect("missing position reserves");
                    let pair = position
                        .data
                        .expect("missing position")
                        .phi
                        .expect("missing position trading function")
                        .pair
                        .expect("missing trading function pair");
                    planner.position_withdraw(
                        position_id,
                        reserves.try_into().expect("invalid reserves"),
                        pair.try_into().expect("invalid pair"),
                    );
                }

                let final_plan = planner
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(final_plan).await?;
            }
            TxCmd::Position(PositionCmd::Withdraw {
                source,
                position_id,
                fee_tier,
            }) => {
                let mut client = DexQueryServiceClient::new(app.pd_channel().await?);

                // Fetch the information regarding the position from the view service.
                let position = client
                    .liquidity_position_by_id(LiquidityPositionByIdRequest {
                        position_id: Some(PositionId::from(*position_id)),
                    })
                    .await?
                    .into_inner();

                let reserves = position
                    .data
                    .clone()
                    .expect("missing position metadata")
                    .reserves
                    .expect("missing position reserves");
                let pair = position
                    .data
                    .expect("missing position")
                    .phi
                    .expect("missing position trading function")
                    .pair
                    .expect("missing trading function pair");

                let plan = Planner::new(OsRng)
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .position_withdraw(*position_id, reserves.try_into()?, pair.try_into()?)
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Position(PositionCmd::RewardClaim {}) => todo!(),
            TxCmd::Position(PositionCmd::Replicate(replicate_cmd)) => {
                replicate_cmd.exec(app).await?;
            }
        }
        Ok(())
    }
}
