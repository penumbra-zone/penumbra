use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
#[cfg(feature = "component")]
use penumbra_sdk_dex::component::SwapDataRead;
use penumbra_sdk_fee::component::StateReadExt as _;
use penumbra_sdk_governance::StateReadExt as _;
use penumbra_sdk_proto::DomainType;
use penumbra_sdk_sct::component::clock::EpochRead;
use penumbra_sdk_sct::component::tree::{SctManager as _, SctRead};
use penumbra_sdk_shielded_pool::component::NoteManager as _;
use tracing::instrument;

use crate::{state_key, CompactBlock};

#[async_trait]
pub trait CompactBlockManager: StateWrite {
    /// Finish an SCT block and use the resulting roots to finalize the current `CompactBlock`.
    async fn finish_block(&mut self) -> Result<()> {
        self.finalize_compact_block(false).await
    }

    /// Finish an SCT block and epoch and use the resulting roots to finalize the current `CompactBlock`.
    async fn finish_epoch(&mut self) -> Result<()> {
        self.finalize_compact_block(true).await
    }
}

impl<T: StateWrite + ?Sized> CompactBlockManager for T {}

#[async_trait]
trait Inner: StateWrite {
    #[instrument(skip_all)]
    async fn finalize_compact_block(&mut self, end_epoch: bool) -> Result<()> {
        use penumbra_sdk_shielded_pool::component::StateReadExt as _;
        // Find out what our block height is (this is set even during the genesis block)
        let height = self
            .get_block_height()
            .await
            .expect("height of block is always set");
        tracing::debug!(?height, ?end_epoch, "finishing compact block");

        // This will report a "false positive" if parameters were scheduled to be changed but
        // the update failed. We don't really care if a client re-fetches parameters in that case.
        let mut app_parameters_updated = self
            .param_changes_for_height(height)
            .await
            .expect("should be able to check for param changes")
            .is_some();
        // Force app_parameters_updated to true for the genesis compactblock.
        app_parameters_updated = app_parameters_updated || height == 0;

        // Check to see if the gas prices have changed, and include them in the compact block
        // if they have (this is signaled by `penumbra_sdk_fee::StateWriteExt::put_gas_prices`):
        let (gas_prices, alt_gas_prices) = if self.gas_prices_changed() || height == 0 {
            (
                Some(
                    self.get_gas_prices()
                        .await
                        .context("could not get gas prices")?,
                ),
                self.get_alt_gas_prices()
                    .await
                    .context("could not get alt gas prices")?,
            )
        } else {
            (None, Vec::new())
        };

        let fmd_parameters = if height == 0 {
            Some(
                self.get_current_fmd_parameters()
                    .await
                    .context("could not get FMD parameters")?,
            )
        } else {
            None
        };

        // Check to see if a governance proposal has started, and mark this fact if so.
        let proposal_started = self.proposal_started();

        // End the block in the SCT and record the block root, epoch root if applicable, and the SCT
        // itself, storing the resultant block and epoch root if applicable in the compact block.
        let (block_root, epoch_root) = self
            .end_sct_block(end_epoch)
            .await
            .context("could not end SCT block")?;

        // Pull out all the pending state payloads (note and swap)
        let note_payloads = self
            .pending_note_payloads()
            .into_iter()
            // Strip the sources of transaction IDs
            .map(|(pos, note, source)| (pos, (note, source.stripped()).into()));
        let rolled_up_payloads = self
            .pending_rolled_up_payloads()
            .into_iter()
            .map(|(pos, commitment)| (pos, commitment.into()));
        let swap_payloads = self
            .pending_swap_payloads()
            .into_iter()
            // Strip the sources of transaction IDs
            .map(|(pos, swap, source)| (pos, (swap, source.stripped()).into()));

        // Sort the payloads by position and put them in the compact block
        let mut state_payloads = note_payloads
            .chain(rolled_up_payloads)
            .chain(swap_payloads)
            .collect::<Vec<_>>();
        state_payloads.sort_by_key(|(pos, _)| *pos);
        let state_payloads = state_payloads
            .into_iter()
            .map(|(_, payload)| payload)
            .collect();

        // Gather the swap outputs
        let swap_outputs = self.pending_batch_swap_outputs().into_iter().collect();

        // Add all the pending nullifiers to the compact block
        let nullifiers = self.pending_nullifiers().into_iter().collect();

        //Get the index of the current epoch
        let epoch_index = self
            .get_current_epoch()
            .await
            .expect("epoch is always set")
            .index;

        let compact_block = CompactBlock {
            height,
            state_payloads,
            nullifiers,
            block_root,
            epoch_root,
            proposal_started,
            swap_outputs,
            fmd_parameters,
            app_parameters_updated,
            gas_prices,
            alt_gas_prices,
            epoch_index,
        };

        self.nonverifiable_put_raw(
            state_key::compact_block(height).into_bytes(),
            compact_block.encode_to_vec(),
        );

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> Inner for T {}
