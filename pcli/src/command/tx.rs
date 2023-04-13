use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Result};
use ark_ff::UniformRand;
use decaf377::Fr;
use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use penumbra_app::stake::rate::RateData;
use penumbra_crypto::{
    asset,
    dex::{
        lp::{position::Position, Reserves, TradingFunction},
        DirectedTradingPair,
    },
    keys::AddressIndex,
    memo::MemoPlaintext,
    stake::{DelegationToken, IdentityKey, Penalty, UnbondingToken},
    transaction::Fee,
    Amount, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{
    client::v1alpha1::{
        EpochByHeightRequest, LiquidityPositionByIdRequest, ProposalInfoRequest,
        ProposalInfoResponse, ProposalRateDataRequest, ValidatorPenaltyRequest,
    },
    core::dex::v1alpha1::PositionId,
};
use penumbra_transaction::{
    action::Ics20Withdrawal,
    plan::{SwapClaimPlan, UndelegateClaimPlan},
    proposal::ProposalToml,
    vote::Vote,
};
use penumbra_view::ViewClient;
use penumbra_wallet::plan::{self, Planner};
use rand_core::OsRng;

use crate::App;

mod proposal;
use proposal::ProposalCmd;

mod liquidity_position;
use liquidity_position::PositionCmd;

use self::liquidity_position::OrderCmd;

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
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0", display_order = 200)]
        fee: u64,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
        /// Optional. Set the transaction's memo field to the provided text.
        #[clap(long)]
        memo: Option<String>,
    },
    /// Deposit stake into a validator's delegation pool.
    #[clap(display_order = 200)]
    Delegate {
        /// The identity key of the validator to delegate to.
        #[clap(long, display_order = 100)]
        to: String,
        /// The amount of stake to delegate.
        amount: String,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0", display_order = 200)]
        fee: u64,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
    },
    /// Withdraw stake from a validator's delegation pool.
    #[clap(display_order = 200)]
    Undelegate {
        /// The amount of delegation tokens to undelegate.
        amount: String,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0", display_order = 200)]
        fee: u64,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
    },
    /// Claim any undelegations that have finished unbonding.
    #[clap(display_order = 200)]
    UndelegateClaim {
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
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
        /// The denomination to swap the input into.
        #[clap(long, display_order = 100)]
        into: String,
        /// The transaction fee (paid in upenumbra).
        ///
        /// A swap generates two transactions; the fee will be split equally over both.
        #[clap(long, default_value = "0", display_order = 200)]
        fee: u64,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
    },
    /// Vote on a governance proposal in your role as a delegator (see also: `pcli validator vote`).
    #[clap(display_order = 400)]
    Vote {
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0", global = true, display_order = 200)]
        fee: u64,
        /// Only spend funds and vote with staked delegation tokens originally received by the given
        /// account.
        #[clap(long, default_value = "0", global = true, display_order = 300)]
        source: u32,
        #[clap(subcommand)]
        vote: VoteCmd,
    },
    /// Submit or withdraw a governance proposal.
    #[clap(display_order = 500, subcommand)]
    Proposal(ProposalCmd),
    /// Deposit funds into the DAO.
    #[clap(display_order = 600)]
    DaoDeposit {
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0", global = true, display_order = 200)]
        fee: u64,
        /// The amounts to send, written as typed values 1.87penumbra, 12cubes, etc.
        values: Vec<String>,
        /// Only spend funds originally received by the given account.
        #[clap(long, default_value = "0", display_order = 300)]
        source: u32,
    },
    /// Manage liquidity positions.
    #[clap(display_order = 500, subcommand)]
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

    /// Perform an ICS-20 withdrawal.
    #[clap(display_order = 250)]
    Withdraw {
        // fully qualified address, eg; cosmos1grgelyng2v6v3t8z87wu3sxgt9m5s03xvslewd@cosmoshub-4
        to: String,
        denom: String, //TODO: should we pull this out of amount
        amount: String,
        source_channel: String,

        #[clap(long, default_value = "0", display_order = 100)]
        timeout_height: u64,
        #[clap(long, default_value = "0", display_order = 150)]
        timeout_timestamp: u64,

        #[clap(long, default_value = "0", display_order = 200)]
        source: u32,
    },
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
            TxCmd::DaoDeposit { .. } => false,
            TxCmd::Position(lp_cmd) => lp_cmd.offline(),
            TxCmd::Withdraw { .. } => false,
        }
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            TxCmd::Send {
                values,
                to,
                fee,
                source: from,
                memo,
            } => {
                // Parse all of the values provided.
                let values = values
                    .iter()
                    .map(|v| v.parse())
                    .collect::<Result<Vec<Value>, _>>()?;
                let fee = Fee::from_staking_token_amount((*fee).into());
                let to = to
                    .parse()
                    .map_err(|_| anyhow::anyhow!("address is invalid"))?;

                let memo = memo.as_ref().map(|m| {
                    let memo_ephemeral_address =
                        app.fvk.ephemeral_address(OsRng, AddressIndex::new(*from)).0;

                    MemoPlaintext {
                        sender: memo_ephemeral_address,
                        text: m.clone(),
                    }
                });

                let plan = plan::send(
                    app.fvk.account_group_id(),
                    app.view.as_mut().unwrap(),
                    OsRng,
                    &values,
                    fee,
                    to,
                    AddressIndex::new(*from),
                    memo,
                )
                .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::DaoDeposit {
                fee,
                values,
                source,
            } => {
                let values = values
                    .iter()
                    .map(|v| v.parse())
                    .collect::<Result<Vec<Value>, _>>()?;
                let fee = Fee::from_staking_token_amount((*fee).into());

                let mut planner = Planner::new(OsRng);
                planner.fee(fee);
                for value in values {
                    planner.dao_deposit(value);
                }
                let plan = planner
                    .plan(
                        app.view.as_mut().unwrap(),
                        app.fvk.account_group_id(),
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Sweep => loop {
                let specific_client = app.specific_client().await?;
                let plans = plan::sweep(
                    app.fvk.account_group_id(),
                    app.view.as_mut().unwrap(),
                    OsRng,
                    specific_client,
                )
                .await?;
                let num_plans = plans.len();

                for (i, plan) in plans.into_iter().enumerate() {
                    println!("building sweep {i} of {num_plans}");
                    let tx = app.build_transaction(plan).await?;
                    app.submit_transaction_unconfirmed(tx).await?;
                }
                if num_plans == 0 {
                    println!("finished sweeping");
                    break;
                } else {
                    println!("awaiting confirmations...");
                    tokio::time::sleep(std::time::Duration::from_secs(6)).await;
                }
            },
            TxCmd::Swap {
                input,
                into,
                fee,
                source,
            } => {
                let input = input.parse::<Value>()?;
                let into = asset::REGISTRY.parse_unit(into.as_str()).base();

                // Since the swap command consists of two transactions (the swap and the swap claim),
                // the fee is split equally over both for now.
                let swap_fee = Fee::from_staking_token_amount((fee / 2).into());
                let swap_claim_fee = Fee::from_staking_token_amount((fee / 2).into());

                let fvk = app.fvk.clone();

                // If a source address was specified, use it for the swap, otherwise,
                // use the default address.
                let (claim_address, _dtk_d) =
                    fvk.incoming().payment_address(AddressIndex::new(*source));

                let mut planner = Planner::new(OsRng);
                planner.fee(swap_fee);
                planner.swap(input, into, swap_claim_fee.clone(), claim_address)?;

                let account_group_id = app.fvk.account_group_id();
                let plan = planner
                    .plan(app.view(), account_group_id, AddressIndex::new(*source))
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
                    .swap_by_commitment(account_group_id, swap_plaintext.swap_commitment())
                    .await?;

                let asset_cache = app.view().assets().await?;

                let pro_rata_outputs = swap_record.output_data.pro_rata_outputs((
                    swap_plaintext.delta_1_i.into(),
                    swap_plaintext.delta_2_i.into(),
                ));
                println!("Swap submitted and batch confirmed!");
                println!(
                    "You will receive outputs of {} and {}. Claiming now...",
                    Value {
                        amount: pro_rata_outputs.0.into(),
                        asset_id: swap_record.output_data.trading_pair.asset_1()
                    }
                    .format(&asset_cache),
                    Value {
                        amount: pro_rata_outputs.1.into(),
                        asset_id: swap_record.output_data.trading_pair.asset_2()
                    }
                    .format(&asset_cache),
                );

                let params = app.view.as_mut().unwrap().chain_params().await?;

                let account_group_id = app.fvk.account_group_id();

                let mut planner = Planner::new(OsRng);
                let plan = planner
                    .swap_claim(SwapClaimPlan {
                        swap_plaintext,
                        position: swap_record.position,
                        output_data: swap_record.output_data,
                        epoch_duration: params.epoch_duration,
                    })
                    .plan(app.view(), account_group_id, AddressIndex::new(*source))
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
                let fee = Fee::from_staking_token_amount((*fee).into());

                let plan = plan::delegate(
                    app.fvk.account_group_id(),
                    app.view.as_mut().unwrap(),
                    OsRng,
                    rate_data,
                    unbonded_amount.into(),
                    fee,
                    AddressIndex::new(*source),
                )
                .await?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Undelegate {
                amount,
                fee,
                source,
            } => {
                let delegation_value @ Value {
                    amount: _,
                    asset_id,
                } = amount.parse::<Value>()?;
                let fee = Fee::from_staking_token_amount((*fee).into());

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

                let mut client = app.specific_client().await?;
                let rate_data: RateData = client
                    .next_validator_rate(tonic::Request::new(from.into()))
                    .await?
                    .into_inner()
                    .try_into()?;

                let mut planner = Planner::new(OsRng);

                let plan = planner
                    .fee(fee)
                    .undelegate(delegation_value.amount, rate_data)
                    .plan(
                        app.view.as_mut().unwrap(),
                        app.fvk.account_group_id(),
                        AddressIndex::new(*source),
                    )
                    .await
                    .context("can't build undelegate plan")?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::UndelegateClaim { fee } => {
                let fee = Fee::from_staking_token_amount((*fee).into());

                let account_group_id = app.fvk.account_group_id(); // this should be optional? or saved in the client statefully?

                let mut specific_client = app.specific_client().await?;
                let mut oblivious_client = app.oblivious_client().await?;
                let view: &mut dyn ViewClient = app.view.as_mut().unwrap();

                let params = view.chain_params().await?;
                let current_height = view.status(account_group_id).await?.sync_height;
                let current_epoch = oblivious_client
                    .epoch_by_height(EpochByHeightRequest {
                        height: current_height,
                    })
                    .await?
                    .into_inner()
                    .epoch
                    .unwrap();
                let asset_cache = view.assets().await?;

                // Query the view client for the list of undelegations that are ready to be claimed.
                // We want to claim them into the same address index that currently holds the tokens.
                let notes = view
                    .unspent_notes_by_address_and_asset(account_group_id)
                    .await?;

                for (address_index, notes_by_asset) in notes.into_iter() {
                    for (token, notes) in
                        notes_by_asset.into_iter().filter_map(|(asset_id, notes)| {
                            // Filter for notes that are unbonding tokens.
                            let denom = asset_cache.get(&asset_id).unwrap().clone();
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

                        let penalty: Penalty = specific_client
                            .validator_penalty(tonic::Request::new(ValidatorPenaltyRequest {
                                chain_id: params.chain_id.to_string(),
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
                            })
                            .fee(fee.clone())
                            .plan(
                                app.view.as_mut().unwrap(),
                                app.fvk.account_group_id(),
                                address_index,
                            )
                            .await?;
                        app.build_and_submit_transaction(plan).await?;
                    }
                }
            }
            TxCmd::Proposal(ProposalCmd::Submit { file, fee, source }) => {
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
                let fee = Fee::from_staking_token_amount((*fee).into());
                let plan = plan::proposal_submit(
                    app.fvk.account_group_id(),
                    app.view.as_mut().unwrap(),
                    OsRng,
                    proposal,
                    fee,
                    AddressIndex::new(*source),
                )
                .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Proposal(ProposalCmd::Withdraw {
                proposal_id,
                fee,
                reason,
                source,
            }) => {
                let fee = Fee::from_staking_token_amount((*fee).into());
                let plan = plan::proposal_withdraw(
                    app.fvk.account_group_id(),
                    app.view.as_mut().unwrap(),
                    OsRng,
                    *proposal_id,
                    reason.clone(),
                    fee,
                    AddressIndex::new(*source),
                )
                .await?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Proposal(ProposalCmd::Template { file, kind }) => {
                let chain_params = app.view().chain_params().await?;

                // Find out what the latest proposal ID is so we can include the next ID in the template:
                let mut client = app.specific_client().await?;
                let next_proposal_id: u64 = client
                    .key_proto(penumbra_app::governance::state_key::next_proposal_id())
                    .await?;

                let toml_template: ProposalToml = kind
                    .template_proposal(&chain_params, next_proposal_id)?
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
                fee,
                proposal_id,
                source,
            }) => {
                use penumbra_app::governance::state_key;
                use penumbra_transaction::proposal;

                let fee = Fee::from_staking_token_amount((*fee).into());

                let mut client = app.specific_client().await?;
                let state: proposal::State = client
                    .key_domain(state_key::proposal_state(*proposal_id))
                    .await?;

                let outcome = match state {
                    proposal::State::Voting => anyhow::bail!(
                        "proposal {} is still voting, so the deposit cannot yet be claimed",
                        proposal_id
                    ),
                    proposal::State::Withdrawn { reason: _ } => {
                        anyhow::bail!("proposal {} has been withdrawn but voting has not yet concluded, so the deposit cannot yet be claimed", proposal_id);
                    }
                    proposal::State::Finished { outcome } => outcome.map(|_| ()),
                    proposal::State::Claimed { outcome: _ } => {
                        anyhow::bail!("proposal {} has already been claimed", proposal_id)
                    }
                };

                let deposit_amount: Amount = client
                    .key_domain(state_key::proposal_deposit_amount(*proposal_id))
                    .await?;

                let plan = Planner::new(OsRng)
                    .proposal_deposit_claim(*proposal_id, deposit_amount, outcome)
                    .fee(fee)
                    .plan(
                        app.view.as_mut().unwrap(),
                        app.fvk.account_group_id(),
                        AddressIndex::new(*source),
                    )
                    .await?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Vote { vote, fee, source } => {
                let (proposal_id, vote): (u64, Vote) = (*vote).into();

                // Before we vote on the proposal, we have to gather some information about it so
                // that we can prepare our vote:
                // - the start height, so we can select the votable staked notes to vote with
                // - the start position, so we can submit the appropriate public `start_position`
                //   input for stateless proof verification
                // - the rate data for every validator at the start of the proposal, so we can
                //   convert staked notes into voting power and mint the correct amount of voting
                //   receipt tokens to ourselves

                let mut client = app.specific_client().await?;
                let ProposalInfoResponse {
                    start_block_height,
                    start_position,
                } = client
                    .proposal_info(ProposalInfoRequest {
                        chain_id: app.view().chain_params().await?.chain_id,
                        proposal_id,
                    })
                    .await?
                    .into_inner();
                let start_position = start_position.into();

                let mut rate_data_stream = client
                    .proposal_rate_data(ProposalRateDataRequest {
                        chain_id: app.view().chain_params().await?.chain_id,
                        proposal_id,
                    })
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

                let fee = Fee::from_staking_token_amount((*fee).into());

                let plan = Planner::new(OsRng)
                    .delegator_vote(
                        proposal_id,
                        start_block_height,
                        start_position,
                        start_rate_data,
                        vote,
                    )
                    .fee(fee)
                    .plan(
                        app.view.as_mut().unwrap(),
                        app.fvk.account_group_id(),
                        AddressIndex::new(*source),
                    )
                    .await?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Position(PositionCmd::Order(OrderCmd::Buy {
                buy_order,
                spread,
                fee,
                source,
            })) => {
                let fee = Fee::from_staking_token_amount((*fee).into());

                // Use DirectedTradingPair to get the canonical trading pair associated with the direction of the buy order.
                let trading_pair =
                    DirectedTradingPair::new(buy_order.price.asset_id, buy_order.desired.asset_id)
                        .to_canonical();

                // This represents an "ask" in the order book, where bids are placed in the asset type without initial reserves.
                //
                // The [`Position`] constructor expects the ordering of [`Reserves`] to match the ordering of the assets in the [`TradingPair`].
                //
                // When opening a liquidity position, the initial reserves will only be set for one asset.
                let (p, q, reserves) = if trading_pair.asset_1() == buy_order.desired.asset_id {
                    (
                        buy_order.price.amount.clone() * 1_000_000u64.into(),
                        1_000_000u64.into(),
                        Reserves {
                            // r1 will be set to 0 units of the asset being bought
                            r1: Amount::zero(),
                            // and r2 will be set to the amount of the asset being sold that would be needed to buy the desired amount of the asset being bought
                            // (divided by 1_000_000 to correct the scaling)
                            r2: (buy_order.price.amount * buy_order.desired.amount)
                                / 10u128.pow(6).into(),
                        },
                    )
                } else {
                    (
                        1_000_000u64.into(),
                        buy_order.price.amount.clone() * 1_000_000u64.into(),
                        Reserves {
                            // r1 will be set to the amount of the asset being sold that would be needed to buy the desired amount of the asset being bought
                            // (divided by 1_000_000 to correct the scaling)
                            r1: (buy_order.price.amount * buy_order.desired.amount)
                                / 10u128.pow(6).into(),
                            // and r2 will be set to 0 units of the asset being bought
                            r2: Amount::zero(),
                        },
                    )
                };

                // `spread` is another name for `fee`, which is at most 10_000 bps.
                if *spread > 10_000 {
                    anyhow::bail!("spread parameter must be at most 10_000bps (i.e. 100%)");
                }

                let trading_function = TradingFunction::new(trading_pair, *spread, p, q);

                let position = Position::new(OsRng, trading_function);
                let plan = Planner::new(OsRng)
                    .position_open(position, reserves)
                    .fee(fee)
                    .plan(
                        app.view.as_mut().unwrap(),
                        app.fvk.account_group_id(),
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Withdraw {
                to,
                amount,
                denom,
                timeout_height,
                timeout_timestamp,
                source_channel,
                source,
            } => {
                // TODO: should we be using a standard address parser here?
                let to_components = to.split('@').collect::<Vec<_>>();
                let destination_chain_address = to_components[0];
                let destination_chain_id = to_components[1];

                let fee = Fee::from_staking_token_amount(Amount::zero());
                let (ephemeral_return_address, _) = app
                    .fvk
                    .ephemeral_address(OsRng, AddressIndex::from(*source));
                let account_group_id = app.fvk.account_group_id();
                let current_height = app.view().status(account_group_id).await?.sync_height;

                // get the current time on the local machine
                let current_time_u64 = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                let mut timeout_height = *timeout_height;
                if timeout_height == 0u64 {
                    // add two days to height, assuming 6 blocks per minute
                    timeout_height = current_height + 28800u64;
                }
                let mut timeout_timestamp = *timeout_timestamp;
                if timeout_timestamp == 0u64 {
                    // add 2 days to current time
                    timeout_timestamp = current_time_u64 + 172800u64;
                }

                let denom = asset::REGISTRY.parse_denom(denom).unwrap();
                let amount = Amount::try_from(amount.clone()).unwrap();

                let withdrawal = Ics20Withdrawal {
                    destination_chain_id: destination_chain_id.to_string(),
                    destination_chain_address: destination_chain_address.to_string(),
                    denom,
                    amount,
                    timeout_height,
                    timeout_time: timeout_timestamp,
                    return_address: ephemeral_return_address,
                    source_channel: ChannelId::from_str(source_channel)?,
                    source_port: PortId::from_str("transfer")?,
                };

                let plan = Planner::new(OsRng)
                    .ics20_withdrawal(withdrawal)
                    .fee(fee)
                    .plan(
                        app.view.as_mut().unwrap(),
                        app.fvk.account_group_id(),
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Position(PositionCmd::Order(OrderCmd::Sell {
                sell_order,
                spread,
                fee,
                source,
            })) => {
                let fee = Fee::from_staking_token_amount((*fee).into());

                // Use DirectedTradingPair to get the canonical trading pair associated with the direction of the sell order.
                // The sell order represents the desire to move from the `selling` asset to the `price` asset.
                let trading_pair = DirectedTradingPair::new(
                    sell_order.selling.asset_id,
                    sell_order.price.asset_id,
                )
                .to_canonical();

                // This represents an "ask" in the order book, where bids are placed in the asset type without initial reserves.
                //
                // The [`Position`] constructor expects the ordering of [`Reserves`] to match the ordering of the assets in the [`TradingPair`].
                //
                // When opening a liquidity position, the initial reserves will only be set for one asset.
                let (p, q, reserves) = if trading_pair.asset_1() == sell_order.selling.asset_id {
                    (
                        1_000_000u64.into(),
                        sell_order.price.amount.clone() * 1_000_000u64.into(),
                        Reserves {
                            // r1 will be set to the amount of the asset being sold
                            r1: sell_order.selling.amount,
                            // r2 will be set to 0 units of the asset being bought
                            r2: Amount::zero(),
                        },
                    )
                } else {
                    (
                        sell_order.price.amount.clone() * 1_000_000u64.into(),
                        1_000_000u64.into(),
                        Reserves {
                            // r1 will be set to 0 units of the asset being bought
                            r1: Amount::zero(),
                            // r2 will be set to the amount of the asset being sold
                            r2: sell_order.selling.amount,
                        },
                    )
                };

                // `spread` is another name for `fee`, which is at most 10_000 bps.
                if *spread > 10_000 {
                    anyhow::bail!("spread parameter must be at most 10_000bps (i.e. 100%)");
                }

                let trading_function = TradingFunction::new(trading_pair, *spread, p, q);

                let position = Position::new(OsRng, trading_function);
                let plan = Planner::new(OsRng)
                    .position_open(position, reserves)
                    .fee(fee)
                    .plan(
                        app.view.as_mut().unwrap(),
                        app.fvk.account_group_id(),
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Position(PositionCmd::Close {
                position_id,
                fee,
                source,
            }) => {
                let fee = Fee::from_staking_token_amount((*fee).into());

                let plan = Planner::new(OsRng)
                    .position_close(*position_id)
                    .fee(fee)
                    .plan(
                        app.view.as_mut().unwrap(),
                        app.fvk.account_group_id(),
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Position(PositionCmd::Withdraw {
                fee,
                source,
                position_id,
            }) => {
                let mut specific_client = app.specific_client().await?;

                let view: &mut dyn ViewClient = app.view.as_mut().unwrap();
                let params = view.chain_params().await?;

                // Fetch the information regarding the position from the view service.
                let position = specific_client
                    .liquidity_position_by_id(LiquidityPositionByIdRequest {
                        chain_id: params.chain_id.to_string(),
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
                    .expect("missing position metadata")
                    .position
                    .expect("missing position")
                    .phi
                    .expect("missing position trading function")
                    .pair
                    .expect("missing trading function pair");

                let fee = Fee::from_staking_token_amount((*fee).into());

                let plan = Planner::new(OsRng)
                    .position_withdraw(*position_id, reserves.try_into()?, pair.try_into()?)
                    .fee(fee)
                    .plan(
                        app.view.as_mut().unwrap(),
                        app.fvk.account_group_id(),
                        AddressIndex::new(*source),
                    )
                    .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Position(PositionCmd::RewardClaim {}) => todo!(),
        }
        Ok(())
    }
}
