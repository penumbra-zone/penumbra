use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_chain::component::StateReadExt as _;
use penumbra_dex::component::StateReadExt;
use penumbra_dex::component::SwapManager as _;
use penumbra_fee::component::StateReadExt as _;
use penumbra_governance::StateReadExt as _;
use penumbra_proto::DomainType;
use penumbra_sct::component::SctManager as _;
use penumbra_sct::component::StateReadExt as _;
use penumbra_shielded_pool::component::NoteManager as _;
use tracing::instrument;

use crate::{state_key, CompactBlock};

#[async_trait]
pub trait CompactBlockManager: StateWrite {
    /// Finish an SCT block and use the resulting roots to finalize the current `CompactBlock`.
    async fn finish_block(&mut self, app_parameters_updated: bool) -> Result<()> {
        self.finalize_compact_block(false, app_parameters_updated)
            .await
    }

    /// Finish an SCT block and epoch and use the resulting roots to finalize the current `CompactBlock`.
    async fn finish_epoch(&mut self, app_parameters_updated: bool) -> Result<()> {
        self.finalize_compact_block(true, app_parameters_updated)
            .await
    }
}

impl<T: StateWrite + ?Sized> CompactBlockManager for T {}

#[async_trait]
trait Inner: StateWrite {
    #[instrument(skip_all)]
    async fn finalize_compact_block(
        &mut self,
        end_epoch: bool,
        mut app_parameters_updated: bool,
    ) -> Result<()> {
        use penumbra_shielded_pool::component::StateReadExt as _;
        // Find out what our block height is (this is set even during the genesis block)
        let height = self
            .get_block_height()
            .await
            .expect("height of block is always set");
        tracing::debug!(?height, ?end_epoch, "finishing compact block");

        // Force app_parameters_updated to true for the genesis compactblock.
        app_parameters_updated = app_parameters_updated || height == 0;

        // Check to see if the gas prices have changed, and include them in the compact block
        // if they have (this is signaled by `penumbra_fee::StateWriteExt::put_gas_prices`):
        let gas_prices = if self.gas_prices_changed() || height == 0 {
            Some(
                self.get_gas_prices()
                    .await
                    .context("could not get gas prices")?,
            )
        } else {
            None
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
        };

        self.nonverifiable_put_raw(
            state_key::compact_block(height).into_bytes(),
            compact_block.encode_to_vec(),
        );

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> Inner for T {}
