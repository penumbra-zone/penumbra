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
}

impl StakeCmd {
    pub fn needs_sync(&self) -> bool {
        true
    }

    pub async fn exec(&self, _opt: &Opt, state: &ClientState) -> Result<()> {
        match self {
            StakeCmd::Delegate { .. } => {
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
        }

        Ok(())
    }
}
