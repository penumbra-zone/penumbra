use anyhow::{anyhow, Result};
use penumbra_crypto::Value;
use penumbra_proto::thin_wallet::{thin_wallet_client::ThinWalletClient, ValidatorRateRequest};
use penumbra_stake::{Epoch, IdentityKey, RateData, STAKING_TOKEN_ASSET_ID};
use penumbra_wallet::ClientState;
use structopt::StructOpt;

use crate::Opt;

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
        /// The identity key of the validator to withdraw delegation from.
        #[structopt(long)]
        from: String,
        /// The amount of stake to delegate.
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

    pub async fn exec(&self, opt: &Opt, state: &ClientState) -> Result<()> {
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

                // FIXME! need some kind of structure for recording chain
                // parameters - connected with having protos for genesis data
                // (though not all genesis data is chain parameters)
                let epoch_duration = 10;
                let current_epoch =
                    Epoch::from_height(state.last_block_height().unwrap() as u64, epoch_duration);

                let mut client = ThinWalletClient::connect(format!(
                    "http://{}:{}",
                    opt.node, opt.thin_wallet_port
                ))
                .await?;

                let rate_data: RateData = client
                    .validator_rate(tonic::Request::new(ValidatorRateRequest {
                        identity_key: Some(to.into()),
                        epoch_index: current_epoch.index,
                    }))
                    .await?
                    .into_inner()
                    .try_into()?;

                let delegation_amount = rate_data.delegation_amount(unbonded_amount);

                // Steps:
                //
                // - check that we have at least `amount` of staking token to spend
                // - construct a spend description that releases unbonded_amount + fee
                // - construct a delegate description that consumes unbonded_amount staking token, produces delegation_amount of delegation token
                // - construct an output description that records the new delegation tokens
                // - construct an output description that records the change
                // - finalize transaction
                // - submit a transaction to the rpc endpoint

                // outsource computation to staking crate? wallet crate?
                // overlap with tx command?

                todo!()
            }
            StakeCmd::Undelegate { .. } => {
                todo!()
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
