use anyhow::{anyhow, Context, Result};
use penumbra_crypto::Value;
use penumbra_proto::thin_wallet::ValidatorRateRequest;
use penumbra_stake::{DelegationToken, Epoch, IdentityKey, RateData, STAKING_TOKEN_ASSET_ID};
use rand_core::OsRng;
use structopt::StructOpt;

use crate::{ClientStateFile, Opt};

#[derive(Debug, StructOpt)]
pub enum StakeCmd {
    /// Deposit stake into a validator's delegation pool.
    Delegate {
        /// The identity key of the validator to delegate to.
        #[structopt(long)]
        to: String,
        /// The amount of stake to delegate.
        amount: String,
        /// The transaction fee (paid in upenumbra).
        #[structopt(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[structopt(long)]
        source: Option<u64>,
    },
    /// Withdraw stake from a validator's delegation pool.
    Undelegate {
        /// The amount of delegation tokens to undelegate.
        amount: String,
        /// The transaction fee (paid in upenumbra).
        #[structopt(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[structopt(long)]
        source: Option<u64>,
    },
    /// Redelegate stake from one validator's delegation pool to another.
    Redelegate {
        /// The identity key of the validator to withdraw delegation from.
        #[structopt(long)]
        from: String,
        /// The identity key of the validator to delegate to.
        #[structopt(long)]
        to: String,
        /// The amount of stake to delegate.
        amount: String,
        /// The transaction fee (paid in upenumbra).
        #[structopt(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[structopt(long)]
        source: Option<u64>,
    },
    /// Display this wallet's delegations and their value.
    Show,
    /// Display all of the validators participating in the chain.
    ListValidators {
        /// Whether to show validators that are not currently part of the consensus set.
        #[structopt(long)]
        show_inactive: bool,
    },
}

impl StakeCmd {
    pub fn needs_sync(&self) -> bool {
        true
    }

    pub async fn exec(&self, opt: &Opt, state: &mut ClientStateFile) -> Result<()> {
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

                let current_epoch = Epoch::from_height(
                    state.last_block_height().unwrap() as u64,
                    state.chain_params().unwrap().epoch_duration,
                );
                let next_epoch = current_epoch.next();

                let mut client = opt.thin_wallet_client().await?;

                let rate_data: RateData = client
                    .validator_rate(tonic::Request::new(ValidatorRateRequest {
                        identity_key: Some(to.into()),
                        epoch_index: next_epoch.index,
                    }))
                    .await?
                    .into_inner()
                    .try_into()?;

                let transaction =
                    state.build_delegate(&mut OsRng, rate_data, unbonded_amount, *fee, *source)?;

                opt.submit_transaction(&transaction).await?;
                // Only commit the state if the transaction was submitted successfully,
                // so that we don't store pending notes that will never appear on-chain.
                state.commit()?;
            }
            StakeCmd::Undelegate {
                amount,
                fee,
                source,
            } => {
                let Value {
                    amount: delegation_amount,
                    asset_id,
                } = amount.parse::<Value>()?;

                let delegation_token: DelegationToken = state
                    .asset_cache()
                    .get(&asset_id)
                    .ok_or_else(|| anyhow::anyhow!("unknown asset id {}", asset_id))?
                    .clone()
                    .try_into()
                    .context("could not parse supplied denomination as a delegation token")?;

                let from = delegation_token.validator();

                let current_epoch = Epoch::from_height(
                    state.last_block_height().unwrap() as u64,
                    state.chain_params().unwrap().epoch_duration,
                );
                let next_epoch = current_epoch.next();

                let mut client = opt.thin_wallet_client().await?;

                let rate_data: RateData = client
                    .validator_rate(tonic::Request::new(ValidatorRateRequest {
                        identity_key: Some(from.into()),
                        epoch_index: next_epoch.index,
                    }))
                    .await?
                    .into_inner()
                    .try_into()?;

                let transaction = state.build_undelegate(
                    &mut OsRng,
                    rate_data,
                    delegation_amount,
                    *fee,
                    *source,
                )?;

                opt.submit_transaction(&transaction).await?;
                // Only commit the state if the transaction was submitted successfully,
                // so that we don't store pending notes that will never appear on-chain.
                state.commit()?;
            }
            StakeCmd::Redelegate { .. } => {
                todo!()
            }
            StakeCmd::Show => {
                todo!()
            }
            StakeCmd::ListValidators { .. } => {
                todo!()
            }
        }

        Ok(())
    }
}
