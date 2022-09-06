use crate::index;

use super::*;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Earliest {
    #[serde(flatten, default)]
    earliest_position: EarliestPosition,
    #[serde(default)]
    earliest_forgotten: Forgotten,
    #[serde(default)]
    next: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct EarliestPosition {
    epoch: u16,
    #[serde(flatten, default)]
    earliest_block_position: EarliestBlockPosition,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
struct EarliestBlockPosition {
    block: u16,
    #[serde(default)]
    commitment: u16,
}

impl From<EarliestPosition> for Position {
    fn from(earliest_position: EarliestPosition) -> Self {
        u64::from(index::within::Tree {
            epoch: earliest_position.epoch.into(),
            block: earliest_position.earliest_block_position.block.into(),
            commitment: earliest_position.earliest_block_position.commitment.into(),
        })
        .into()
    }
}

impl Earliest {
    pub fn earlier_than(&self, tree: &Tree) -> bool {
        let position = if let Some(position) = tree.position() {
            position
        } else {
            // If there is no position, the tree is full, so the only way to be earlier than the
            // tree is for the forgotten index to be earlier
            return if self.next {
                tree.forgotten() > self.earliest_forgotten
            } else {
                tree.forgotten() >= self.earliest_forgotten
            };
        };

        // Otherwise, one of the forgotten index or the position must be earlier (strictly
        // earlier if the next parameter is specified)
        if self.next {
            position > self.earliest_position.into() || tree.forgotten() > self.earliest_forgotten
        } else {
            position >= self.earliest_position.into() || tree.forgotten() >= self.earliest_forgotten
        }
    }
}
