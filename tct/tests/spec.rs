use std::collections::VecDeque;

use penumbra_tct::internal::active::Insert;

pub type Tier<T> = VecDeque<Insert<T>>;

#[path = "spec/block.rs"]
pub mod block;
#[path = "spec/epoch.rs"]
pub mod epoch;
#[path = "spec/eternity.rs"]
pub mod eternity;
#[path = "spec/tree.rs"]
mod tree;

pub enum InsertError {
    Full,
    EpochFull,
    EpochForgotten,
    BlockFull,
    BlockForgotten,
}

use tree::Tree;

#[test]
fn test() {
    let builder = eternity::Builder::default();
    builder.build();
}
