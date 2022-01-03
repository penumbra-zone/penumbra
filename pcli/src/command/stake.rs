use anyhow::Result;
use penumbra_stake::IdentityKey;
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

    pub async fn exec(&self, _opt: &Opt, state: &ClientState) -> Result<()> {
        match self {
            StakeCmd::Delegate {
                to,
                amount,
                fee,
                source,
            } => {
                let unbonded_amount = amount;
                // Steps:
                //
                // - parse amount, check that it's staking token
                // - parse the to address, check that it's a validator identity
                // - check that we have at least `amount` of staking token to spend
                // - obtain the rate data for the current epoch
                // - use the rata data to compute the correct amount of delegation token
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
