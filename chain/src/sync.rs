use bytes::Bytes;
use std::convert::TryFrom;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use penumbra_crypto::{FieldExt, Nullifier};
use penumbra_proto::{chain as pb, Protobuf};
use penumbra_transaction::action::output;

// Domain type for CompactBlock.
// Contains the minimum data needed to update client state.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(try_from = "pb::CompactBlock", into = "pb::CompactBlock")]
pub struct CompactBlock {
    pub height: u64,
    // Output bodies describing new notes.
    pub outputs: Vec<output::Body>,
    // Nullifiers identifying spent notes.
    pub nullifiers: Vec<Nullifier>,
}

impl Protobuf<pb::CompactBlock> for CompactBlock {}

impl From<CompactBlock> for pb::CompactBlock {
    fn from(cb: CompactBlock) -> Self {
        pb::CompactBlock {
            height: cb.height,
            outputs: cb.outputs.into_iter().map(Into::into).collect(),
            nullifiers: cb
                .nullifiers
                .into_iter()
                .map(|v| Bytes::copy_from_slice(&v.0.to_bytes()))
                .collect(),
        }
    }
}

impl TryFrom<pb::CompactBlock> for CompactBlock {
    type Error = anyhow::Error;

    fn try_from(value: pb::CompactBlock) -> Result<Self, Self::Error> {
        Ok(CompactBlock {
            height: value.height,
            outputs: value
                .outputs
                .into_iter()
                .map(output::Body::try_from)
                .collect::<Result<Vec<output::Body>>>()?,
            nullifiers: value
                .nullifiers
                .into_iter()
                .map(|v| Nullifier::try_from(&*v))
                .collect::<Result<Vec<Nullifier>>>()?,
        })
    }
}
