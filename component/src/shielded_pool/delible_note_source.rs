use penumbra_chain::NoteSource;
use penumbra_proto::{chain as pb, Protobuf};

/// A [`NoteSource`] which can be deleted (we use this in the state because the JMT does not support
/// deletion).
#[derive(Debug, Clone)]
pub enum DelibleNoteSource {
    Deleted,
    Present(NoteSource),
}

impl Default for DelibleNoteSource {
    fn default() -> Self {
        DelibleNoteSource::Deleted
    }
}

impl From<Option<NoteSource>> for DelibleNoteSource {
    fn from(v: Option<NoteSource>) -> Self {
        match v {
            Some(v) => DelibleNoteSource::Present(v),
            None => DelibleNoteSource::Deleted,
        }
    }
}

impl From<DelibleNoteSource> for Option<NoteSource> {
    fn from(v: DelibleNoteSource) -> Self {
        match v {
            DelibleNoteSource::Deleted => None,
            DelibleNoteSource::Present(v) => Some(v),
        }
    }
}

impl Protobuf<pb::DelibleNoteSource> for DelibleNoteSource {}

impl TryFrom<pb::DelibleNoteSource> for DelibleNoteSource {
    type Error = anyhow::Error;

    fn try_from(v: pb::DelibleNoteSource) -> Result<Self, Self::Error> {
        match v.source {
            None => Ok(DelibleNoteSource::Deleted),
            Some(v) => Ok(DelibleNoteSource::Present(v.try_into()?)),
        }
    }
}

impl From<DelibleNoteSource> for pb::DelibleNoteSource {
    fn from(v: DelibleNoteSource) -> Self {
        match v {
            DelibleNoteSource::Deleted => pb::DelibleNoteSource { source: None },
            DelibleNoteSource::Present(v) => pb::DelibleNoteSource {
                source: Some(v.into()),
            },
        }
    }
}
