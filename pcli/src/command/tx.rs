use std::{fs::File, io::Write};

use anyhow::{anyhow, Context, Result};
use ark_ff::UniformRand;
use decaf377::Fr;
use penumbra_chain::Epoch;
use penumbra_component::stake::rate::RateData;
use penumbra_crypto::{
    asset,
    dex::BatchSwapOutputData,
    stake::{DelegationToken, IdentityKey, Penalty, UnbondingToken},
    transaction::Fee,
    Address, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{
    client::v1alpha1::{BatchSwapOutputDataRequest, KeyValueRequest, ValidatorPenaltyRequest},
    Protobuf,
};
use penumbra_transaction::{action::Proposal, plan::UndelegateClaimPlan};
use penumbra_view::ViewClient;
use penumbra_wallet::plan::{self, Planner};
use rand_core::OsRng;

use crate::App;

mod proposal;
use proposal::ProposalCmd;

#[derive(Debug, clap::Subcommand)]
pub enum TxCmd {
    /// Send funds to a Penumbra address.
    #[clap(display_order = 100)]
    Send {
        /// The destination address to send funds to.
        #[clap(long)]
        to: String,
        /// The amounts to send, written as typed values 1.87penumbra, 12cubes, etc.
        values: Vec<String>,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[clap(long)]
        source: Option<u64>,
        /// Optional. Set the transaction's memo field to the provided text.
        #[clap(long)]
        memo: Option<String>,
    },
    /// Deposit stake into a validator's delegation pool.
    #[clap(display_order = 200)]
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
    #[clap(display_order = 200)]
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
    /// Claim any undelegations that have finished unbonding.
    #[clap(display_order = 200)]
    UndelegateClaim {
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
    },
    /// Redelegate stake from one validator's delegation pool to another.
    #[clap(display_order = 200)]
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
        #[clap(long)]
        into: String,
        /// The transaction fee (paid in upenumbra).
        ///
        /// A swap generates two transactions; the fee will be split equally over both.
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[clap(long)]
        source: Option<u64>,
    },
    /// Submit or withdraw a governance proposal.
    #[clap(display_order = 400, subcommand)]
    Proposal(ProposalCmd),
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
            TxCmd::Redelegate { .. } => false,
            TxCmd::Proposal(proposal_cmd) => proposal_cmd.offline(),
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
                let fee = Fee::from_staking_token_amount((*fee as u64).into());
                let to = to
                    .parse()
                    .map_err(|_| anyhow::anyhow!("address is invalid"))?;

                let plan = plan::send(
                    &app.fvk,
                    app.view.as_mut().unwrap(),
                    OsRng,
                    &values,
                    fee,
                    to,
                    *from,
                    memo.clone(),
                )
                .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Sweep => loop {
                let specific_client = app.specific_client().await?;
                let plans =
                    plan::sweep(&app.fvk, app.view.as_mut().unwrap(), OsRng, specific_client)
                        .await?;
                let num_plans = plans.len();

                for (i, plan) in plans.into_iter().enumerate() {
                    println!("building sweep {} of {}", i, num_plans);
                    let tx = app.build_transaction(plan).await?;
                    app.submit_transaction_unconfirmed(&tx).await?;
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
                let swap_fee = fee / 2;
                let swap_claim_fee = fee / 2;

                let swap_plan = plan::swap(
                    &app.fvk,
                    app.view.as_mut().unwrap(),
                    OsRng,
                    input,
                    into,
                    Fee::from_staking_token_amount(swap_fee.into()),
                    Fee::from_staking_token_amount(swap_claim_fee.into()),
                    *source,
                )
                .await?;
                let swap_plan_inner = swap_plan
                    .swap_plans()
                    .next()
                    .expect("expected swap plan")
                    .clone();

                // Submit the `Swap` transaction.
                app.build_and_submit_transaction(swap_plan).await?;

                // Wait for detection of the note commitment containing the Swap NFT.
                let account_id = app.fvk.hash();
                let note_commitment = swap_plan_inner.swap_body(&app.fvk).swap_nft.note_commitment;
                // Find the swap NFT note associated with the swap plan.
                let swap_nft_record = tokio::time::timeout(
                    std::time::Duration::from_secs(20),
                    app.view()
                        .await_note_by_commitment(account_id, note_commitment),
                )
                .await
                .context("timeout waiting to detect commitment of submitted transaction")?
                .context("error while waiting for detection of submitted transaction")?;

                // Now that the note commitment is detected, we can submit the `SwapClaim` transaction.
                let swap_plaintext = swap_plan_inner.swap_plaintext;

                // Fetch the batch swap output data associated with the block height
                // and trading pair of the swap action.
                //
                // This batch swap output data comes from the client, it's necessary because
                // the client has to encrypt the SwapPlaintext, however the validators *must*
                // validate that the BatchSwapOutputData is correct when processing the SwapClaim!
                let mut client = app.specific_client().await?;
                let output_data: BatchSwapOutputData = client
                    .batch_swap_output_data(BatchSwapOutputDataRequest {
                        height: swap_nft_record.height_created,
                        trading_pair: Some(swap_plaintext.trading_pair.into()),
                    })
                    .await?
                    .into_inner()
                    .try_into()
                    .context("cannot parse batch swap output data")?;

                let view_client: &mut dyn ViewClient = app.view.as_mut().unwrap();
                let asset_cache = view_client.assets().await?;

                let pro_rata_outputs = output_data.pro_rata_outputs((
                    swap_plaintext.delta_1_i.into(),
                    swap_plaintext.delta_2_i.into(),
                ));
                println!("Swap submitted and batch confirmed!");
                println!(
                    "Swap was: {}",
                    if output_data.success {
                        "successful"
                    } else {
                        "unsuccessful"
                    }
                );
                println!(
                    "You will receive outputs of {} and {}. Claiming now...",
                    Value {
                        amount: pro_rata_outputs.0.into(),
                        asset_id: output_data.trading_pair.asset_1()
                    }
                    .format(&asset_cache),
                    Value {
                        amount: pro_rata_outputs.1.into(),
                        asset_id: output_data.trading_pair.asset_2()
                    }
                    .format(&asset_cache),
                );

                let claim_plan = plan::swap_claim(
                    &app.fvk,
                    app.view.as_mut().unwrap(),
                    OsRng,
                    swap_plaintext,
                    swap_nft_record.note,
                    swap_nft_record.position,
                    output_data,
                )
                .await?;

                // Submit the `SwapClaim` transaction. TODO: should probably wait for the output notes
                // of a SwapClaim to sync.
                app.build_and_submit_transaction(claim_plan).await?;
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
                let fee = Fee::from_staking_token_amount((*fee as u64).into());

                let plan = plan::delegate(
                    &app.fvk,
                    app.view.as_mut().unwrap(),
                    OsRng,
                    rate_data,
                    unbonded_amount.into(),
                    fee,
                    *source,
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
                let fee = Fee::from_staking_token_amount((*fee as u64).into());

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

                let params = app.view.as_mut().unwrap().chain_params().await?;

                let end_epoch_index = rate_data.epoch_index + params.unbonding_epochs;

                let mut planner = Planner::new(OsRng);

                let plan = planner
                    .fee(fee)
                    .undelegate(delegation_value.amount, rate_data, end_epoch_index)
                    .plan(app.view.as_mut().unwrap(), &app.fvk, source.map(Into::into))
                    .await
                    .context("can't build undelegate plan")?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::UndelegateClaim { fee } => {
                let fee = Fee::from_staking_token_amount((*fee as u64).into());

                let account_id = app.fvk.hash(); // this should be optional? or saved in the client statefully?

                let mut specific_client = app.specific_client().await?;
                let view: &mut dyn ViewClient = app.view.as_mut().unwrap();

                let params = view.chain_params().await?;
                let current_height = view.status(account_id).await?.sync_height;
                let current_epoch = Epoch::from_height(current_height, params.epoch_duration);
                let asset_cache = view.assets().await?;

                // Query the view client for the list of undelegations that are ready to be claimed.
                // We want to claim them into the same address index that currently holds the tokens.
                let notes = view.unspent_notes_by_address_and_asset(account_id).await?;
                std::mem::drop(view);

                for (address_index, notes_by_asset) in notes.into_iter() {
                    for (token, notes) in notes_by_asset
                        .into_iter()
                        .filter_map(|(asset_id, notes)| {
                            // Filter for notes that are unbonding tokens.
                            let denom = asset_cache.get(&asset_id).unwrap().clone();
                            match UnbondingToken::try_from(denom) {
                                Ok(token) => Some((token, notes)),
                                Err(_) => None,
                            }
                        })
                        .filter_map(|(token, notes)| {
                            // Filter for notes that are ready to be claimed.
                            if token.end_epoch_index() <= current_epoch.index {
                                Some((token, notes))
                            } else {
                                println!(
                                    "skipping {} because it is not yet ready to be claimed",
                                    token.denom().default_unit(),
                                );
                                None
                            }
                        })
                    {
                        println!("claiming {}", token.denom().default_unit());
                        let validator_identity = token.validator();
                        let start_epoch_index = token.start_epoch_index();
                        let end_epoch_index = token.end_epoch_index();

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
                                end_epoch_index,
                                penalty,
                                unbonding_amount,
                                balance_blinding: Fr::rand(&mut OsRng),
                            })
                            .fee(fee.clone())
                            .plan(app.view.as_mut().unwrap(), &app.fvk, Some(address_index))
                            .await?;
                        app.build_and_submit_transaction(plan).await?;
                    }
                }
            }
            TxCmd::Redelegate { .. } => {
                println!("Sorry, this command is not yet implemented");
            }
            TxCmd::Proposal(ProposalCmd::Submit { file, fee, source }) => {
                let proposal: Proposal = serde_json::from_reader(File::open(file)?)?;
                let fee = Fee::from_staking_token_amount((*fee as u64).into());
                let plan = plan::proposal_submit(
                    &app.fvk,
                    app.view.as_mut().unwrap(),
                    OsRng,
                    proposal,
                    fee,
                    *source,
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
                // Download the refund address for the proposal to be withdrawn, so we can derive
                // the address index (if it's one of ours), which is used to form the randomizer for
                // the signature
                let chain_id = app.view().chain_params().await?.chain_id;
                let mut client = app.specific_client().await?;
                // TODO: convert this into an actual query method?
                // Alternatively, store proposals locally, avoiding the remote query?
                let deposit_refund_address = Address::decode(
                    &client
                        .key_value(KeyValueRequest {
                            chain_id,
                            key: penumbra_component::governance::state_key::proposal_deposit_refund_address(
                                *proposal_id,
                            ),
                            proof: false,
                        })
                        .await?
                        .into_inner()
                        .value[..],
                )?;

                let fee = Fee::from_staking_token_amount((*fee as u64).into());
                let plan = plan::proposal_withdraw(
                    &app.fvk,
                    app.view.as_mut().unwrap(),
                    OsRng,
                    *proposal_id,
                    deposit_refund_address,
                    reason.clone(),
                    fee,
                    *source,
                )
                .await?;

                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Proposal(ProposalCmd::Template { file, kind }) => {
                let chain_id = app.view().chain_params().await?.chain_id;
                let template = kind.template_proposal(chain_id);

                if let Some(file) = file {
                    File::create(file)
                        .with_context(|| format!("cannot create file {:?}", file))?
                        .write_all(&serde_json::to_vec_pretty(&template)?)
                        .context("could not write file")?;
                } else {
                    println!("{}", serde_json::to_string_pretty(&template)?);
                }
            }
            TxCmd::Proposal(ProposalCmd::Vote {
                proposal_id: _,
                vote: _,
                fee: _,
                source: _,
            }) => {
                println!("Sorry, delegator voting is not yet implemented");
                // TODO: fill this in for delegator votes
                // let plan = plan::delegator_vote(
                //     &app.fvk,
                //     &mut app.view,
                //     OsRng,
                //     *proposal_id,
                //     *vote,
                //     *fee,
                //     *source,
                // )
                // .await?;
                // app.build_and_submit_transaction(plan).await?;
            }
        }
        Ok(())
    }
}
