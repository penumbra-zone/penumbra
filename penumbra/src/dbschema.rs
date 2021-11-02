use std::convert::TryInto;
use std::str::FromStr;

use decaf377::FieldExt;
use tendermint::block::Height;

use penumbra_crypto::merkle::Root;
use penumbra_crypto::Fq;

/// Bridge type between Postgres and Penumbra
#[derive(Debug, sqlx::FromRow)]
pub struct NoteCommitmentTreeAnchor {
    pub id: i32,
    pub height: i64,
    pub anchor: Vec<u8>,
}

/// Type for Penumbra for its representation
/// this type is not conduvcive with the database as-is
/// or at least the author could not find an easier way and
/// hence the bridge type is used above `NoteCommitmentTreeAnchor`
#[derive(Debug, sqlx::FromRow)]
pub struct PenumbraNoteCommitmentTreeAnchor {
    pub id: i32,
    pub height: Height,
    pub anchor: Root,
}

/// Convert between Penumbra and bridge type for DB
impl From<PenumbraNoteCommitmentTreeAnchor> for NoteCommitmentTreeAnchor {
    fn from(p: PenumbraNoteCommitmentTreeAnchor) -> Self {
        NoteCommitmentTreeAnchor {
            id: p.id,
            height: i64::from(p.height),
            // anchor.0 because it is Root(Fq), so we need inner type
            anchor: FieldExt::to_bytes(&p.anchor.0).to_vec(),
        }
    }
}

/// Convert between bridge and Penumbra type for DB
/// We need both because the conversion each way is through
/// a different (read HACKY) route
impl From<NoteCommitmentTreeAnchor> for PenumbraNoteCommitmentTreeAnchor {
    fn from(n: NoteCommitmentTreeAnchor) -> Self {
        let anchor = vec_to_array(n.anchor);
        PenumbraNoteCommitmentTreeAnchor {
            id: n.id,
            height: Height::from_str(n.height.to_string().as_str()).unwrap(), // WARN: they did not have From<i64> but had FromStr so yolo
            anchor: Root(Fq::from_bytes(anchor).unwrap()),
        }
    }
}

// Shamelessly copied from https://stackoverflow.com/questions/29570607/is-there-a-good-way-to-convert-a-vect-to-an-array
fn vec_to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}
