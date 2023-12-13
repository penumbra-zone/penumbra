use penumbra_proto::penumbra::core::component::sct::v1alpha1 as pb;
use penumbra_tct::builder::{block, epoch};

pub fn sct_anchor(height: u64, anchor: &tct::Root) -> pb::EventRootAnchor {
    pb::EventRootAnchor {
        height: Some(height.into()),
        root_anchor: Some(anchor.into()),
    }
}

pub fn sct_epoch_anchor(index: u64, anchor: &epoch::Root) -> pb::EventEpochAnchor {
    pb::EventEpochAnchor {
        index: Some(index.into()),
        epoch_anchor: Some(anchor.into()),
    }
}

pub fn sct_block_anchor(height: u64, anchor: &block::Root) -> pb::EventBlockAnchor {
    pb::EventBlockAnchor {
        height: Some(height.into()),
        block_anchor: Some(anchor.into()),
    }
}
