use penumbra_tct as tct;
use tct::builder::{block, epoch};
use tendermint::abci::{Event, EventAttributeIndexExt};

pub fn sct_anchor(height: u64, anchor: &tct::Root) -> Event {
    Event::new(
        "sct_anchor",
        [
            ("height", height.to_string()).index(),
            ("anchor", anchor.to_string()).index(),
        ],
    )
}

pub fn sct_block_anchor(height: u64, anchor: &block::Root) -> Event {
    Event::new(
        "sct_block_anchor",
        [
            ("height", height.to_string()).index(),
            ("anchor", anchor.to_string()).index(),
        ],
    )
}

pub fn sct_epoch_anchor(index: u64, anchor: &epoch::Root) -> Event {
    Event::new(
        "sct_epoch_anchor",
        [
            ("index", index.to_string()).index(),
            ("anchor", anchor.to_string()).index(),
        ],
    )
}
