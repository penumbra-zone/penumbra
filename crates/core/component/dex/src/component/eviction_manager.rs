use crate::component::StateReadExt;
use crate::{component::position_manager::counter::PositionCounterRead, lp::position};
use futures::{StreamExt as _, TryStreamExt};
use std::collections::BTreeSet;

use crate::state_key::eviction_queue;
use anyhow::Result;
use cnidarium::StateWrite;
use tracing::instrument;

use crate::{component::PositionManager, DirectedTradingPair, TradingPair};

pub(crate) trait EvictionManager: StateWrite {
    /// Evict liquidity positions that are in excess of the trading pair limit.
    ///
    /// # Overview
    /// This method enforce the approximate limit on the number of
    /// positions that can be active for a given trading pair, as defined
    /// by [`max_positions_per_pair`](DexParameters#max_positions_per_pair).
    ///
    /// # Mechanism
    ///
    /// The eviction mechanism functions by inspecting every trading pair which
    /// had LP opened during the block. For each of them, it computes the "excess"
    /// amount of positions `M`, defined as follow:
    /// `M = N - N_max` where N is the number of positions,
    ///                   and N_max is a chain parameter.
    ///
    /// Since a [`TradingPair`] defines two possible directed pairs, we need
    /// to ensure that we don't evict LPs if they provide important liquidity
    /// to at least one direction of the pair.
    ///
    /// To do this effectively, we maintain a liquidity index which orders LPs
    /// by ascending liquidity for each direction of a trading pair. This allow
    /// us to easily fetch the worst M positions for each index, and only evict
    /// overlapping LPs.
    ///
    /// This approach sidestep the problem of adjudicating which position deserve
    /// to be preserved depending on the trading direction, and limit the number
    /// of reads to 2*M. On the other hand, it means that the actual maximum number
    /// of positions per pair is 2*N_max.
    ///
    /// ## Diagram
    ///
    ///                                                      Q_1: A -> B
    ///    ╔════════════╦━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
    ///    ║  Bottom M  ┃  M+1  │  M+2  │    . . .       │  N-1  │   N   ┃
    ///    ╚════════════╩━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
    ///     ▽▼▼▼▼▼▼▼▽▽▼▽
    /// ┌─▶ find overlap
    /// │   ▲▲▲▲△△△▲▲▲▲▲                                     Q_2: B -> A
    /// │  ╔════════════╦━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
    /// │  ║  Bottom M  ┃  M+1  │  M+2  │    . . .       │  N-1  │   N   ┃
    /// │  ╚════════════╩━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
    /// │    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━▶
    /// │                                                         Ordered by inventory
    /// │
    /// │         ▽▼▼▼▼▼▼▼▽▽▼▽
    /// └───────▶      ∩       Across both indices, the
    ///           ▲▲▲▲△△△▲▲▲▲▲ bottom M positions overlap
    ///                        k times where 0 <= k <= M
    ///                        for N < 2*N_max
    ///
    #[instrument(skip_all, err, level = "trace")]
    async fn evict_positions(&mut self) -> Result<()> {
        let hot_pairs: BTreeSet<TradingPair> = self.get_active_trading_pairs_in_block();
        let max_positions_per_pair = self.get_dex_params().await?.max_positions_per_pair;

        for pair in hot_pairs.iter() {
            let total_positions = self.get_position_count(pair).await;
            let overhead_size = total_positions.saturating_sub(max_positions_per_pair);
            if overhead_size == 0 {
                continue;
            }

            let pair_ab = DirectedTradingPair::new(pair.asset_1(), pair.asset_2());
            let pair_ba = pair_ab.flip();
            let key_ab = eviction_queue::inventory_index::by_trading_pair(&pair_ab);
            let key_ba = eviction_queue::inventory_index::by_trading_pair(&pair_ba);

            let stream_ab = self.nonverifiable_prefix_raw(&key_ab).boxed();
            let stream_ba = self.nonverifiable_prefix_raw(&key_ba).boxed();

            let overhead_ab = stream_ab
                .take(overhead_size as usize)
                .and_then(|(k, _)| async move {
                    let raw_id = eviction_queue::inventory_index::parse_id_from_key(k)?;
                    Ok(position::Id(raw_id))
                })
                .try_collect::<BTreeSet<position::Id>>()
                .await?;

            let overhead_ba = stream_ba
                .take(overhead_size as usize)
                .and_then(|(k, _)| async move {
                    let raw_id = eviction_queue::inventory_index::parse_id_from_key(k)?;
                    Ok(position::Id(raw_id))
                })
                .try_collect::<BTreeSet<position::Id>>()
                .await?;

            let overlap = overhead_ab.intersection(&overhead_ba);

            for id in overlap {
                self.close_position_by_id(id).await?;
            }
        }
        Ok(())
    }
}

impl<T: StateWrite + ?Sized> EvictionManager for T {}
