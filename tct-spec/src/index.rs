//! Functions for constructing indices of commitments within blocks, epochs, and eternities.

use crate::*;

use hash_hasher::HashedMap;
use penumbra_tct as tct;

/// Index a block, producing a mapping from witnessed commitments to their position in the block.
fn block(block: &List<Insert<Commitment>>) -> HashedMap<Commitment, tct::index::within::Block> {
    let mut index = HashedMap::default();
    for (within_block, insert) in block.iter().enumerate() {
        if let Some(commitment) = insert.keep() {
            index.insert(
                commitment,
                tct::index::within::Block::from(within_block as u16),
            );
        }
    }
    index
}

/// Index an epoch, producing a mapping from witnessed commitments to their position in the epoch.
fn epoch(
    epoch: &List<Insert<List<Insert<Commitment>>>>,
) -> HashedMap<Commitment, tct::index::within::Epoch> {
    let mut index = HashedMap::default();
    for (within_epoch, insert) in epoch.iter().enumerate() {
        if let Some(block) = insert.as_ref().keep() {
            index.extend(
                self::block(block)
                    .into_iter()
                    .map(|(commitment, within_block)| {
                        (
                            commitment,
                            tct::index::within::Epoch::from(
                                (within_epoch as u32) << 16 | u32::from(u16::from(within_block)),
                            ),
                        )
                    }),
            );
        }
    }
    index
}

/// Index an eternity, producing a mapping from witnessed commitments to their position in the eternity.
pub fn tree(
    tree: &List<Insert<List<Insert<List<Insert<Commitment>>>>>>,
) -> HashedMap<Commitment, tct::Position> {
    let mut index = HashedMap::default();
    for (within_eternity, insert) in tree.iter().enumerate() {
        if let Some(epoch) = insert.as_ref().keep() {
            index.extend(
                self::epoch(epoch)
                    .into_iter()
                    .map(|(commitment, within_epoch)| {
                        (
                            commitment,
                            tct::Position::from(
                                (within_eternity as u64) << 32 | u64::from(u32::from(within_epoch)),
                            ),
                        )
                    }),
            );
        }
    }
    index
}
