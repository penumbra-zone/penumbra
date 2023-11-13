use anyhow::Result;

use address::AddressCmd;
use balance::BalanceCmd;
use staked::StakedCmd;
use transaction_hashes::TransactionHashesCmd;
use tx::TxCmd;
use wallet_id::WalletIdCmd;

use crate::App;

mod address;
mod balance;
mod staked;
mod wallet_id;

pub mod transaction_hashes;
mod tx;

#[derive(Debug, clap::Subcommand)]
pub enum ViewCmd {
    /// View your wallet id
    WalletId(WalletIdCmd),
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
    /// Displays a transaction's details by hash.
    Tx(TxCmd),
}

impl ViewCmd {
    pub fn offline(&self) -> bool {
        match self {
            ViewCmd::WalletId(wallet_id_cmd) => wallet_id_cmd.offline(),
            ViewCmd::Address(address_cmd) => address_cmd.offline(),
            ViewCmd::Balance(balance_cmd) => balance_cmd.offline(),
            ViewCmd::Staked(staked_cmd) => staked_cmd.offline(),
            ViewCmd::Reset(_) => true,
            ViewCmd::Sync => false,
            ViewCmd::ListTransactionHashes(transactions_cmd) => transactions_cmd.offline(),
            ViewCmd::Tx(tx_cmd) => tx_cmd.offline(),
        }
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        // TODO: refactor view methods to take a single App
        let full_viewing_key = app.config.full_viewing_key.clone();

        match self {
            ViewCmd::WalletId(wallet_id_cmd) => {
                wallet_id_cmd.exec(&full_viewing_key)?;
            }
            ViewCmd::Tx(tx_cmd) => {
                tx_cmd.exec(app).await?;
            }
            ViewCmd::ListTransactionHashes(transactions_cmd) => {
                let view_client = app.view();
                transactions_cmd
                    .exec(&full_viewing_key, view_client)
                    .await?;
            }
            ViewCmd::Sync => {
                // We set needs_sync() -> true, so by this point, we have
                // already synchronized the wallet above, so we can just return.
            }
            ViewCmd::Reset(_reset) => {
                // The wallet has already been reset by a short-circuiting path.
            }
            ViewCmd::Address(address_cmd) => {
                address_cmd.exec(&full_viewing_key)?;
            }
            ViewCmd::Balance(balance_cmd) => {
                let view_client = app.view();
                balance_cmd.exec(&full_viewing_key, view_client).await?;
            }
            ViewCmd::Staked(staked_cmd) => {
                let channel = app.pd_channel().await?;
                let view_client = app.view();
                staked_cmd
                    .exec(&full_viewing_key, view_client, channel)
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
            println!("Deleted view data at {view_path}");
        } else if view_path.exists() {
            anyhow::bail!(
                "Expected view data at {} but found something that is not a file; refusing to delete it",
                view_path
            );
        } else {
            anyhow::bail!(
                "No view data exists at {}, so it cannot be deleted",
                view_path
            );
        }

        Ok(())
    }
}
