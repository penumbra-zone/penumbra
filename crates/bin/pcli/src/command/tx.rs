use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{ensure, Context, Result};
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
use lqt_vote::LqtVoteCmd;
use rand_core::OsRng;
use regex::Regex;

use liquidity_position::PositionCmd;
use penumbra_sdk_asset::{asset, asset::Metadata, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_dex::{
    lp::position::{self, Position, State},
    swap_claim::SwapClaimPlan,
};
use penumbra_sdk_fee::FeeTier;
use penumbra_sdk_governance::{
    proposal::ProposalToml, proposal_state::State as ProposalState, Vote,
};
use penumbra_sdk_keys::{keys::AddressIndex, Address};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::core::app::v1::{
    query_service_client::QueryServiceClient as AppQueryServiceClient, AppParametersRequest,
};
use penumbra_sdk_proto::{
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
            ValidatorPenaltyRequest, ValidatorStatusRequest,
        },
    },
    cosmos::tx::v1beta1::{
        mode_info::{Single, Sum},
        service_client::ServiceClient as CosmosServiceClient,
        AuthInfo as CosmosAuthInfo, BroadcastTxRequest as CosmosBroadcastTxRequest,
        Fee as CosmosFee, ModeInfo, SignerInfo as CosmosSignerInfo, Tx as CosmosTx,
        TxBody as CosmosTxBody,
    },
    noble::forwarding::v1::{ForwardingPubKey, MsgRegisterAccount},
    view::v1::GasPricesRequest,
    Message, Name as _,
};
use penumbra_sdk_shielded_pool::Ics20Withdrawal;
use penumbra_sdk_stake::{
    rate::RateData,
    validator::{self},
};
use penumbra_sdk_stake::{
    DelegationToken, IdentityKey, Penalty, UnbondingToken, UndelegateClaimPlan,
};
use penumbra_sdk_transaction::{gas::swap_claim_gas_cost, Transaction};
use penumbra_sdk_view::{SpendableNoteRecord, ViewClient};
use penumbra_sdk_wallet::plan::{self, Planner};
use proposal::ProposalCmd;
use tonic::transport::{Channel, ClientTlsConfig};
use url::Url;

use crate::command::tx::auction::AuctionCmd;
use crate::App;
use clap::Parser;

mod auction;
mod liquidity_position;
mod lqt_vote;
mod proposal;
mod replicate;

/// The planner can fail to build a large transaction, so
/// pcli splits apart the number of positions to close/withdraw
/// in the [`PositionCmd::CloseAll`]/[`PositionCmd::WithdrawAll`] commands.
const POSITION_CHUNK_SIZE: usize = 30;

#[derive(Debug, Parser)]
pub struct TxCmdWithOptions {
    /// If present, a file to save the transaction to instead of broadcasting it
    #[clap(long)]
    pub offline: Option<PathBuf>,
    #[clap(subcommand)]
    pub cmd: TxCmd,
}

impl TxCmdWithOptions {
    /// Determine if this command requires a network sync before it executes.
    pub fn offline(&self) -> bool {
        self.cmd.offline()
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        app.save_transaction_here_instead = self.offline.clone();
        self.cmd.exec(app).await
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum TxCmd {
    /// Auction related commands.
    #[clap(display_order = 600, subcommand)]
    Auction(AuctionCmd),
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
        #[clap(short, long, default_value_t)]
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
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Delegate to many validators in a single transaction.
    #[clap(display_order = 200)]
    DelegateMany {
        /// A path to a CSV file of (validator identity, UM amount) pairs.
        ///
        /// The amount is in UM, not upenumbra.
        #[clap(long, display_order = 100)]
        csv_path: String,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
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
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Claim any undelegations that have finished unbonding.
    #[clap(display_order = 200)]
    UndelegateClaim {
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
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
        #[clap(short, long, default_value_t)]
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
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Submit or withdraw a governance proposal.
    #[clap(display_order = 500, subcommand)]
    Proposal(ProposalCmd),
    /// Deposit funds into the Community Pool.
    #[clap(display_order = 600)]
    CommunityPoolDeposit {
        /// The amounts to send, written as typed values 1.87penumbra, 12cubes, etc.
        #[clap(min_values = 1, required = true)]
        values: Vec<String>,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
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
        /// Optional. Set the IBC ICS-20 packet memo field to the provided text.
        #[clap(long)]
        memo: Option<String>,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
        /// Whether to use a transparent address (bech32, 32-byte) for
        /// the return address in the withdrawal.
        /// Required for some chains for a successful acknowledgement.
        #[clap(long)]
        use_transparent_address: bool,
    },
    #[clap(display_order = 970)]
    /// Register a Noble forwarding account.
    RegisterForwardingAccount {
        /// The Noble node to submit the registration transaction to.
        #[clap(long)]
        noble_node: Url,
        /// The Noble IBC channel to use for forwarding.
        #[clap(long)]
        channel: String,
        /// The Penumbra address or address index to receive forwarded funds.
        #[clap(long)]
        address_or_index: String,
        /// Whether or not to use an ephemeral address.
        #[clap(long)]
        ephemeral: bool,
    },
    /// Broadcast a saved transaction to the network
    #[clap(display_order = 1000)]
    Broadcast {
        /// The transaction to be broadcast
        transaction: PathBuf,
    },
    #[clap(display_order = 700)]
    LqtVote(LqtVoteCmd),
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
            TxCmd::DelegateMany { .. } => false,
            TxCmd::Undelegate { .. } => false,
            TxCmd::UndelegateClaim { .. } => false,
            TxCmd::Vote { .. } => false,
            TxCmd::Proposal(proposal_cmd) => proposal_cmd.offline(),
            TxCmd::CommunityPoolDeposit { .. } => false,
            TxCmd::Position(lp_cmd) => lp_cmd.offline(),
            TxCmd::Withdraw { .. } => false,
            TxCmd::Auction(_) => false,
            TxCmd::Broadcast { .. } => false,
            TxCmd::RegisterForwardingAccount { .. } => false,
            TxCmd::LqtVote(cmd) => cmd.offline(),
        }
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        // TODO: use a command line flag to determine the fee token,
        // and pull the appropriate GasPrices out of this rpc response,
        // the rest should follow
        // TODO: fetching this here means that no tx commands
        // can be run in offline mode, which is a bit annoying
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
                    .parse::<Address>()
                    .map_err(|_| anyhow::anyhow!("address is invalid"))?;

                let mut planner = Planner::new(OsRng);

                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());
                for value in values.iter().cloned() {
                    planner.output(value, to.clone());
                }
                let plan = planner
                    .memo(memo.clone().unwrap_or_default())
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
                let fee_tier: FeeTier = (*fee_tier).into();

                let fvk = app.config.full_viewing_key.clone();

                // If a source address was specified, use it for the swap, otherwise,
                // use the default address.
                let (claim_address, _dtk_d) =
                    fvk.incoming().payment_address(AddressIndex::new(*source));

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices.clone())
                    .set_fee_tier(fee_tier.into());

                // We don't expect much of a drift in gas prices in a few blocks, and the fee tier
                // adjustments should be enough to cover it.
                let estimated_claim_fee = gas_prices
                    .fee(&swap_claim_gas_cost())
                    .apply_tier(fee_tier.into());

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
                        asset_id: swap_record.output_data.trading_pair.asset_1(),
                    }
                    .format(&asset_cache),
                    Value {
                        amount: pro_rata_outputs.1,
                        asset_id: swap_record.output_data.trading_pair.asset_2(),
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
                    .set_fee_tier(fee_tier.into());
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

                let mut stake_client = StakeQueryServiceClient::new(app.pd_channel().await?);
                let rate_data: RateData = stake_client
                    .current_validator_rate(tonic::Request::new(to.into()))
                    .await?
                    .into_inner()
                    .try_into()?;

                let mut sct_client = SctQueryServiceClient::new(app.pd_channel().await?);
                let latest_sync_height = app.view().status().await?.full_sync_height;
                let epoch = sct_client
                    .epoch_by_height(EpochByHeightRequest {
                        height: latest_sync_height,
                    })
                    .await?
                    .into_inner()
                    .epoch
                    .expect("epoch must be available")
                    .into();

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());
                let plan = planner
                    .delegate(epoch, unbonded_amount, rate_data)
                    .plan(app.view(), AddressIndex::new(*source))
                    .await
                    .context("can't plan delegation, try running pcli tx sweep and try again")?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::DelegateMany {
                csv_path,
                source,
                fee_tier,
            } => {
                let mut stake_client = StakeQueryServiceClient::new(app.pd_channel().await?);

                let mut sct_client = SctQueryServiceClient::new(app.pd_channel().await?);
                let latest_sync_height = app.view().status().await?.full_sync_height;
                let epoch = sct_client
                    .epoch_by_height(EpochByHeightRequest {
                        height: latest_sync_height,
                    })
                    .await?
                    .into_inner()
                    .epoch
                    .expect("epoch must be available")
                    .into();

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                let file = File::open(csv_path).context("can't open CSV file")?;
                let mut reader = csv::ReaderBuilder::new()
                    .has_headers(false) // Don't skip any rows
                    .from_reader(file);
                for result in reader.records() {
                    let record = result?;
                    let validator_identity: IdentityKey = record[0].parse()?;

                    let rate_data: RateData = stake_client
                        .current_validator_rate(tonic::Request::new(validator_identity.into()))
                        .await?
                        .into_inner()
                        .try_into()?;

                    let typed_amount_str = format!("{}penumbra", &record[1]);

                    let unbonded_amount = {
                        let Value { amount, asset_id } = typed_amount_str.parse::<Value>()?;
                        if asset_id != *STAKING_TOKEN_ASSET_ID {
                            anyhow::bail!("staking can only be done with the staking token");
                        }
                        amount
                    };

                    planner.delegate(epoch, unbonded_amount, rate_data);
                }

                let plan = planner
                    .plan(app.view(), AddressIndex::new(*source))
                    .await
                    .context("can't plan delegation, try running pcli tx sweep and try again")?;

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

                let mut stake_client = StakeQueryServiceClient::new(app.pd_channel().await?);
                let rate_data: RateData = stake_client
                    .current_validator_rate(tonic::Request::new(from.into()))
                    .await?
                    .into_inner()
                    .try_into()?;

                let mut sct_client = SctQueryServiceClient::new(app.pd_channel().await?);
                let latest_sync_height = app.view().status().await?.full_sync_height;
                let epoch = sct_client
                    .epoch_by_height(EpochByHeightRequest {
                        height: latest_sync_height,
                    })
                    .await?
                    .into_inner()
                    .epoch
                    .expect("epoch must be available")
                    .into();

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                let plan = planner
                    .undelegate(epoch, delegation_value.amount, rate_data)
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
                let asset_cache = view.assets().await?;

                // Query the view client for the list of undelegations that are ready to be claimed.
                // We want to claim them into the same address index that currently holds the tokens.
                let notes = view.unspent_notes_by_address_and_asset().await?;

                let notes: Vec<(
                    AddressIndex,
                    Vec<(UnbondingToken, Vec<SpendableNoteRecord>)>,
                )> = notes
                    .into_iter()
                    .map(|(address_index, notes_by_asset)| {
                        let mut filtered_notes: Vec<(UnbondingToken, Vec<SpendableNoteRecord>)> =
                            notes_by_asset
                                .into_iter()
                                .filter_map(|(asset_id, notes)| {
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
                                .collect();

                        filtered_notes.sort_by_key(|(token, _)| token.unbonding_start_height());

                        (address_index, filtered_notes)
                    })
                    .collect();

                for (address_index, notes_by_asset) in notes.into_iter() {
                    for (token, notes) in notes_by_asset.into_iter() {
                        println!("claiming {}", token.denom().default_unit());

                        let validator_identity = token.validator();
                        let unbonding_start_height = token.unbonding_start_height();

                        let mut app_client = AppQueryServiceClient::new(channel.clone());
                        let mut stake_client = StakeQueryServiceClient::new(channel.clone());
                        let mut sct_client = SctQueryServiceClient::new(channel.clone());

                        let min_block_delay = app_client
                            .app_parameters(AppParametersRequest {})
                            .await?
                            .into_inner()
                            .app_parameters
                            .expect("app parameters must be available")
                            .stake_params
                            .expect("stake params must be available")
                            .unbonding_delay;

                        // Fetch the validator pool's state at present:
                        let bonding_state = stake_client
                            .validator_status(ValidatorStatusRequest {
                                identity_key: Some(validator_identity.into()),
                            })
                            .await?
                            .into_inner()
                            .status
                            .context("unable to get validator status")?
                            .bonding_state
                            .expect("bonding state must be available")
                            .try_into()
                            .expect("valid bonding state");

                        let upper_bound_block_delay = unbonding_start_height + min_block_delay;

                        // We have to be cautious to compute the penalty over the exact range of epochs
                        // because we could be processing old unbonding tokens that are bound to a validator
                        // that transitioned to a variety of states, incurring penalties that do not apply
                        // to these tokens.
                        // We can replace this with a single gRPC call to the staking component.
                        // For now, this is sufficient.
                        let unbonding_height = match bonding_state {
                            validator::BondingState::Bonded => upper_bound_block_delay,
                            validator::BondingState::Unbonding { unbonds_at_height } => {
                                if unbonds_at_height > unbonding_start_height {
                                    unbonds_at_height.min(upper_bound_block_delay)
                                } else {
                                    current_height
                                }
                            }
                            validator::BondingState::Unbonded => current_height,
                        };

                        // if the unbonding height is in the future we clamp to the current height:
                        let unbonding_height = unbonding_height.min(current_height);

                        let start_epoch_index = sct_client
                            .epoch_by_height(EpochByHeightRequest {
                                height: unbonding_start_height,
                            })
                            .await
                            .expect("can get epoch by height")
                            .into_inner()
                            .epoch
                            .context("unable to get epoch for unbonding start height")?
                            .index;

                        let end_epoch_index = sct_client
                            .epoch_by_height(EpochByHeightRequest {
                                height: unbonding_height,
                            })
                            .await
                            .expect("can get epoch by height")
                            .into_inner()
                            .epoch
                            .context("unable to get epoch for unbonding end height")?
                            .index;

                        let penalty: Penalty = stake_client
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

                        let plan = planner
                            .undelegate_claim(UndelegateClaimPlan {
                                validator_identity,
                                unbonding_start_height,
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

                let deposit_amount: Value = deposit_amount.parse()?;
                ensure!(
                    deposit_amount.asset_id == *STAKING_TOKEN_ASSET_ID,
                    "deposit amount must be in staking token"
                );

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());
                let plan = planner
                    .proposal_submit(proposal, deposit_amount.amount)
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
                        app.view(),
                        AddressIndex::new(*source),
                        proposal_id,
                        vote,
                        start_block_height,
                        start_position,
                        start_rate_data,
                    )
                    .await?
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
                let positions = order.as_position(&asset_cache, OsRng)?;
                tracing::info!(?positions);
                for position in &positions {
                    println!("Position id: {}", position.id());
                }

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier(order.fee_tier().into());

                for position in positions {
                    planner.position_open(position);
                }

                let plan = planner
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
                memo,
                fee_tier,
                use_transparent_address,
            } => {
                let destination_chain_address = to;

                let ephemeral_return_address = if *use_transparent_address {
                    let ivk = app.config.full_viewing_key.incoming();

                    ivk.transparent_address()
                        .parse::<Address>()
                        .expect("we round-trip from a valid transparent address")
                } else {
                    app.config
                        .full_viewing_key
                        .ephemeral_address(OsRng, AddressIndex::from(*source))
                        .0
                };

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
                let current_time_ns = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_nanos() as u64;

                let mut timeout_timestamp = *timeout_timestamp;
                if timeout_timestamp == 0u64 {
                    // add 2 days to current time
                    timeout_timestamp = current_time_ns + 1.728e14 as u64;
                }

                // round to the nearest 10 minutes
                timeout_timestamp += 600_000_000_000 - (timeout_timestamp % 600_000_000_000);

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
                    use_compat_address: false,
                    ics20_memo: memo.clone().unwrap_or_default(),
                    use_transparent_address: *use_transparent_address,
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
                position_ids,
                source,
                fee_tier,
            }) => {
                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                position_ids.iter().for_each(|position_id| {
                    planner.position_close(*position_id);
                });

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
                    .owned_position_ids(Some(position::State::Opened), *trading_pair, None)
                    .await?;

                if owned_position_ids.is_empty() {
                    println!("No open positions are available to close.");
                    return Ok(());
                }

                println!(
                    "{} total open positions, closing in {} batches of {}",
                    owned_position_ids.len(),
                    owned_position_ids.len() / POSITION_CHUNK_SIZE + 1,
                    POSITION_CHUNK_SIZE
                );

                let mut planner = Planner::new(OsRng);

                // Close 5 positions in a single transaction to avoid planner failures.
                for positions_to_close_now in owned_position_ids.chunks(POSITION_CHUNK_SIZE) {
                    planner
                        .set_gas_prices(gas_prices)
                        .set_fee_tier((*fee_tier).into());

                    for position_id in positions_to_close_now {
                        // Close the position
                        planner.position_close(*position_id);
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
                    .owned_position_ids(Some(position::State::Closed), *trading_pair, None)
                    .await?;

                if owned_position_ids.is_empty() {
                    println!("No closed positions are available to withdraw.");
                    return Ok(());
                }

                println!(
                    "{} total closed positions, withdrawing in {} batches of {}",
                    owned_position_ids.len(),
                    owned_position_ids.len() / POSITION_CHUNK_SIZE + 1,
                    POSITION_CHUNK_SIZE,
                );

                let mut client = DexQueryServiceClient::new(app.pd_channel().await?);

                let mut planner = Planner::new(OsRng);

                // Withdraw 5 positions in a single transaction to avoid planner failures.
                for positions_to_withdraw_now in owned_position_ids.chunks(POSITION_CHUNK_SIZE) {
                    planner
                        .set_gas_prices(gas_prices)
                        .set_fee_tier((*fee_tier).into());

                    for position_id in positions_to_withdraw_now {
                        // Withdraw the position

                        // Fetch the information regarding the position from the view service.
                        let position = client
                            .liquidity_position_by_id(LiquidityPositionByIdRequest {
                                position_id: Some((*position_id).into()),
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
                            *position_id,
                            reserves.try_into().expect("invalid reserves"),
                            pair.try_into().expect("invalid pair"),
                            0,
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
            }
            TxCmd::Position(PositionCmd::Withdraw {
                source,
                position_ids,
                fee_tier,
            }) => {
                let mut client = DexQueryServiceClient::new(app.pd_channel().await?);

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                for position_id in position_ids {
                    // Fetch the information regarding the position from the view service.
                    let response = client
                        .liquidity_position_by_id(LiquidityPositionByIdRequest {
                            position_id: Some(PositionId::from(*position_id)),
                        })
                        .await?
                        .into_inner();

                    let position: Position = response
                        .data
                        .expect("missing position")
                        .try_into()
                        .expect("invalid position state");

                    let reserves = position.reserves;
                    let pair = position.phi.pair;
                    let next_seq = match position.state {
                        State::Withdrawn { sequence } => sequence + 1,
                        State::Closed => 0,
                        _ => {
                            anyhow::bail!("position {} is not in a withdrawable state", position_id)
                        }
                    };
                    planner.position_withdraw(
                        *position_id,
                        reserves.try_into()?,
                        pair.try_into()?,
                        next_seq,
                    );
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
            TxCmd::Position(PositionCmd::RewardClaim {}) => {
                unimplemented!("deprecated, remove this")
            }
            TxCmd::Position(PositionCmd::Replicate(replicate_cmd)) => {
                replicate_cmd.exec(app).await?;
            }
            TxCmd::Auction(AuctionCmd::Dutch(auction_cmd)) => {
                auction_cmd.exec(app).await?;
            }
            TxCmd::Broadcast { transaction } => {
                let transaction: Transaction = serde_json::from_slice(&fs::read(transaction)?)?;
                app.submit_transaction(transaction).await?;
            }
            TxCmd::RegisterForwardingAccount {
                noble_node,
                channel,
                address_or_index,
                ephemeral,
            } => {
                let index: Result<u32, _> = address_or_index.parse();
                let fvk = app.config.full_viewing_key.clone();

                let address = if let Ok(index) = index {
                    // address index provided
                    let (address, _dtk) = match ephemeral {
                        false => fvk.incoming().payment_address(index.into()),
                        true => fvk.incoming().ephemeral_address(OsRng, index.into()),
                    };

                    address
                } else {
                    // address or nothing provided
                    let address: Address = address_or_index
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Provided address is invalid."))?;

                    address
                };

                let noble_address = address.noble_forwarding_address(channel);

                println!(
                    "registering Noble forwarding account with address {} to forward to Penumbra address {}...",
                    noble_address, address
                );

                let mut noble_client = CosmosServiceClient::new(
                    Channel::from_shared(noble_node.to_string())?
                        .tls_config(ClientTlsConfig::new().with_webpki_roots())?
                        .connect()
                        .await?,
                );

                let tx = CosmosTx {
                    body: Some(CosmosTxBody {
                        messages: vec![pbjson_types::Any {
                            type_url: MsgRegisterAccount::type_url(),
                            value: MsgRegisterAccount {
                                signer: noble_address.to_string(),
                                recipient: address.to_string(),
                                channel: channel.to_string(),
                            }
                            .encode_to_vec()
                            .into(),
                        }],
                        memo: "".to_string(),
                        timeout_height: 0,
                        extension_options: vec![],
                        non_critical_extension_options: vec![],
                    }),
                    auth_info: Some(CosmosAuthInfo {
                        signer_infos: vec![CosmosSignerInfo {
                            public_key: Some(pbjson_types::Any {
                                type_url: ForwardingPubKey::type_url(),
                                value: ForwardingPubKey {
                                    key: noble_address.bytes(),
                                }
                                .encode_to_vec()
                                .into(),
                            }),
                            mode_info: Some(ModeInfo {
                                // SIGN_MODE_DIRECT
                                sum: Some(Sum::Single(Single { mode: 1 })),
                            }),
                            sequence: 0,
                        }],
                        fee: Some(CosmosFee {
                            amount: vec![],
                            gas_limit: 200000u64,
                            payer: "".to_string(),
                            granter: "".to_string(),
                        }),
                        tip: None,
                    }),
                    signatures: vec![vec![]],
                };
                let r = noble_client
                    .broadcast_tx(CosmosBroadcastTxRequest {
                        tx_bytes: tx.encode_to_vec().into(),
                        // sync
                        mode: 2,
                    })
                    .await?;

                // let r = noble_client
                //     .register_account(MsgRegisterAccount {
                //         signer: noble_address,
                //         recipient: address.to_string(),
                //         channel: channel.to_string(),
                //     })
                //     .await?;

                println!("Noble response: {:?}", r);
            }
            TxCmd::LqtVote(cmd) => cmd.exec(app, gas_prices).await?,
        }

        Ok(())
    }
}
