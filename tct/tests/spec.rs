use std::collections::VecDeque;

use penumbra_tct::internal::active::Insert;

#[path = "spec/block.rs"]
pub mod block;
#[path = "spec/epoch.rs"]
pub mod epoch;
#[path = "spec/eternity.rs"]
pub mod eternity;
#[path = "spec/tree.rs"]
pub mod tree;

pub enum InsertError {
    Full,
    EpochFull,
    EpochForgotten,
    BlockFull,
    BlockForgotten,
}

pub type Tier<T> = VecDeque<Insert<T>>;

#[test]
fn test() {
    let builder = eternity::Builder::default();
    builder.build();
}
