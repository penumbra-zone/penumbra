use penumbra_proto::penumbra::core::component::sct::v1alpha1 as pb;
use penumbra_tct::builder::{block, epoch};

/// Create an event tracking the new global SCT anchor.
pub fn sct_anchor(height: u64, anchor: penumbra_tct::Root) -> pb::EventRootAnchor {
    pb::EventRootAnchor {
        height: height.into(),
        root_anchor: Some(anchor.into()),
    }
}

/// Create an event tracking the new SCT anchor for the epoch subtree.
pub fn sct_epoch_anchor(index: u64, anchor: epoch::Root) -> pb::EventEpochAnchor {
    pb::EventEpochAnchor {
        index: index.into(),
        epoch_anchor: Some(anchor.into()),
    }
}

/// Create an event tracking the new SCT anchor for the block subtree.
pub fn sct_block_anchor(height: u64, anchor: block::Root) -> pb::EventBlockAnchor {
    pb::EventBlockAnchor {
        height: height.into(),
        block_anchor: Some(anchor.into()),
    }
}
