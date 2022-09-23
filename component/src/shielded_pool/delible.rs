use penumbra_chain::NoteSource;
use penumbra_proto::{core::chain::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

/// A thing which can be deleted (we use this in the state because the JMT does not support
/// deletion).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Delible<T> {
    Deleted,
    Present(T),
}

impl<T> Default for Delible<T> {
    fn default() -> Self {
        Delible::Deleted
    }
}

impl<T> From<Option<T>> for Delible<T> {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => Delible::Present(v),
            None => Delible::Deleted,
        }
    }
}

impl<T> From<Delible<T>> for Option<T> {
    fn from(v: Delible<T>) -> Self {
        match v {
            Delible::Deleted => None,
            Delible::Present(v) => Some(v),
        }
    }
}

impl Protobuf<pb::DelibleNoteSource> for Delible<NoteSource> {}

impl TryFrom<pb::DelibleNoteSource> for Delible<NoteSource> {
    type Error = anyhow::Error;

    fn try_from(v: pb::DelibleNoteSource) -> Result<Self, Self::Error> {
        match v.source {
            None => Ok(Delible::Deleted),
            Some(v) => Ok(Delible::Present(v.try_into()?)),
        }
    }
}

impl From<Delible<NoteSource>> for pb::DelibleNoteSource {
    fn from(v: Delible<NoteSource>) -> Self {
        match v {
            Delible::Deleted => pb::DelibleNoteSource { source: None },
            Delible::Present(v) => pb::DelibleNoteSource {
                source: Some(v.into()),
            },
        }
    }
}
