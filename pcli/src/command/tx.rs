use anyhow::Result;
use penumbra_crypto::Value;
use rand_core::OsRng;
use structopt::StructOpt;

use crate::{ClientStateFile, Opt};

#[derive(Debug, StructOpt)]
pub enum TxCmd {
    /// Send transaction to the node.
    Send {
        /// The destination address to send funds to.
        #[structopt(long)]
        to: String,
        /// The amounts to send, written as typed values 1.87penumbra, 12cubes, etc.
        values: Vec<String>,
        /// The transaction fee (paid in upenumbra).
        #[structopt(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[structopt(long)]
        source: Option<u64>,
        /// Optional. Set the transaction's memo field to the provided text.
        #[structopt(long)]
        memo: Option<String>,
    },
}

impl TxCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            TxCmd::Send { .. } => true,
        }
    }

    pub async fn exec(&self, opt: &Opt, state: &mut ClientStateFile) -> Result<()> {
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
                let to = &to
                    .parse()
                    .map_err(|_| anyhow::anyhow!("address is invalid"))?;

                let (transaction, mut remainder) =
                    state.build_send(&mut OsRng, &values, *fee, to, *from, memo.clone())?;

                opt.submit_transaction(&transaction).await?;
                // Only commit the state if the transaction was submitted successfully,
                // so that we don't store pending notes that will never appear on-chain.
                state.commit()?;

                // Handle the remainders of this transaction iteratively until none are left
                while let Some(rem) = remainder {
                    // Construct the next transaction to perform based on the remainder of the
                    // previous transaction.
                    let (transaction, rem) = state.continue_with_remainder(&mut OsRng, rem)?;
                    remainder = rem; // set the remainder for the next iteration to the new remainder

                    // Submit this next transaction
                    opt.submit_transaction(&transaction).await?;
                    state.commit()?;
                }
            }
        }
        Ok(())
    }
}
