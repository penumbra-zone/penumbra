use anyhow::Result;
use penumbra_proto::client::oblivious::CompactBlockRangeRequest;
use tracing::instrument;

use crate::{ClientStateFile, Opt};

#[instrument(skip(opt, state), fields(start_height = state.last_block_height()))]
pub async fn sync(opt: &Opt, state: &mut ClientStateFile) -> Result<()> {
    tracing::info!("starting client sync");
    let mut client = opt.oblivious_client().await?;

    let start_height = state.last_block_height().map(|h| h + 1).unwrap_or(0);
    let mut stream = client
        .compact_block_range(tonic::Request::new(CompactBlockRangeRequest {
            start_height,
            end_height: 0,
            chain_id: state
                .chain_id()
                .ok_or_else(|| anyhow::anyhow!("missing chain_id"))?,
            keep_alive: false,
        }))
        .await?
        .into_inner();

    let mut count = 0;
    while let Some(block) = stream.message().await? {
        state.scan_block(block.try_into()?)?;
        // very basic form of intermediate checkpointing
        count += 1;
        if count % 1000 == 1 {
            state.commit()?;
            tracing::info!(height = ?state.last_block_height().unwrap(), "syncing...");
        }
    }

    state.prune_timeouts();
    state.commit()?;
    tracing::info!(end_height = ?state.last_block_height().unwrap(), "finished sync");
    Ok(())
}
