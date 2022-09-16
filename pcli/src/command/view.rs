use anyhow::Result;

use penumbra_crypto::FullViewingKey;
use penumbra_proto::client::oblivious::oblivious_query_client::ObliviousQueryClient;
use penumbra_view::ViewClient;
use tonic::transport::Channel;

mod balance;
use balance::BalanceCmd;
mod address;
use address::AddressCmd;
mod staked;
use staked::StakedCmd;
pub mod transaction_hashes;
use transaction_hashes::TransactionHashesCmd;

#[derive(Debug, clap::Subcommand)]
pub enum ViewCmd {
    /// View one of your addresses, either by numerical index, or a random ephemeral one.
    Address(AddressCmd),
    /// View your account balances.
    Balance(BalanceCmd),
    /// View your staked delegation tokens.
    Staked(StakedCmd),
    /// Deletes all scanned data and local state, while leaving keys untouched.
    Reset(Reset),
    /// Synchronizes the client, privately scanning the chain state.
    ///
    /// `pcli` syncs automatically prior to any action requiring chain state,
    /// but this command can be used to "pre-sync" before interactive use.
    Sync,
    /// Get transaction hashes and block heights of spendable notes.
    #[clap(visible_alias = "list-tx-hashes")]
    ListTransactionHashes(TransactionHashesCmd),
}

impl ViewCmd {
    pub fn needs_sync(&self) -> bool {
        match self {
            ViewCmd::Address(address_cmd) => address_cmd.needs_sync(),
            ViewCmd::Balance(balance_cmd) => balance_cmd.needs_sync(),
            ViewCmd::Staked(staked_cmd) => staked_cmd.needs_sync(),
            ViewCmd::Reset(_) => false,
            ViewCmd::Sync => true,
            ViewCmd::ListTransactionHashes(transactions_cmd) => transactions_cmd.needs_sync(),
        }
    }

    pub async fn exec(
        &self,
        full_viewing_key: &FullViewingKey,
        view_client: &mut impl ViewClient,
        oblivious_client: &mut ObliviousQueryClient<Channel>,
    ) -> Result<()> {
        match self {
            ViewCmd::ListTransactionHashes(transactions_cmd) => {
                transactions_cmd.exec(full_viewing_key, view_client).await?;
            }
            ViewCmd::Sync => {
                // We set needs_sync() -> true, so by this point, we have
                // already synchronized the wallet above, so we can just return.
            }
            ViewCmd::Reset(_reset) => {
                // The wallet has already been reset by a short-circuiting path.
            }
            ViewCmd::Address(address_cmd) => {
                address_cmd.exec(full_viewing_key)?;
            }
            ViewCmd::Balance(balance_cmd) => {
                balance_cmd.exec(full_viewing_key, view_client).await?;
            }
            ViewCmd::Staked(staked_cmd) => {
                staked_cmd
                    .exec(full_viewing_key, view_client, oblivious_client)
                    .await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, clap::Parser)]
pub struct Reset;

impl Reset {
    pub fn exec(&self, data_path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        tracing::info!("resetting client state");
        let view_path = data_path.as_ref().join(crate::VIEW_FILE_NAME);
        if view_path.is_file() {
            std::fs::remove_file(&view_path)?;
            println!("Deleted view data at {}", view_path);
        } else if view_path.exists() {
            return Err(anyhow::anyhow!(
                "Expected view data at {} but found something that is not a file; refusing to delete it",
                view_path
            ));
        } else {
            return Err(anyhow::anyhow!(
                "No view data exists at {}, so it cannot be deleted",
                view_path
            ));
        }

        Ok(())
    }
}
