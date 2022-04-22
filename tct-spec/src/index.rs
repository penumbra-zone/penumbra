use crate::*;

use hash_hasher::HashedMap;
use penumbra_tct as eternity;

pub fn block(block: &[Insert<Commitment>]) -> HashedMap<Commitment, block::Position> {
    let mut index = HashedMap::default();
    for (within_block, insert) in block.iter().enumerate() {
        if let Some(commitment) = insert.keep() {
            index.insert(commitment, block::Position::from(within_block as u16));
        }
    }
    index
}

pub fn epoch(epoch: &[Insert<Vec<Insert<Commitment>>>]) -> HashedMap<Commitment, epoch::Position> {
    let mut index = HashedMap::default();
    for (within_epoch, insert) in epoch.iter().enumerate() {
        if let Some(block) = insert.as_ref().keep() {
            index.extend(
                self::block(block)
                    .into_iter()
                    .map(|(commitment, within_block)| {
                        (
                            commitment,
                            epoch::Position::from(
                                (within_epoch as u32) << 16 | u32::from(u16::from(within_block)),
                            ),
                        )
                    }),
            );
        }
    }
    index
}

pub fn eternity(
    eternity: &[Insert<Vec<Insert<Vec<Insert<Commitment>>>>>],
) -> HashedMap<Commitment, eternity::Position> {
    let mut index = HashedMap::default();
    for (within_eternity, insert) in eternity.iter().enumerate() {
        if let Some(epoch) = insert.as_ref().keep() {
            index.extend(
                self::epoch(epoch)
                    .into_iter()
                    .map(|(commitment, within_epoch)| {
                        (
                            commitment,
                            eternity::Position::from(
                                (within_eternity as u64) << 32 | u64::from(u32::from(within_epoch)),
                            ),
                        )
                    }),
            );
        }
    }
    index
}
