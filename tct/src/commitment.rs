use decaf377::FieldExt;
use penumbra_proto::{crypto as pb, Protobuf};
use poseidon377::Fq;

/// Commitment to the value of a note.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "pb::NoteCommitment", try_from = "pb::NoteCommitment")]
pub struct Commitment(pub Fq);

impl Protobuf<pb::NoteCommitment> for Commitment {}

#[cfg(test)]
mod test_serde {
    use super::Commitment;

    #[test]
    fn roundtrip_json_zero() {
        let commitment = Commitment::try_from([0; 32]).unwrap();
        let bytes = serde_json::to_vec(&commitment).unwrap();
        println!("{:?}", bytes);
        let deserialized: Commitment = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(commitment, deserialized);
    }

    #[test]
    fn roundtrip_bincode_zero() {
        let commitment = Commitment::try_from([0; 32]).unwrap();
        let bytes = bincode::serialize(&commitment).unwrap();
        println!("{:?}", bytes);
        let deserialized: Commitment = bincode::deserialize(&bytes).unwrap();
        assert_eq!(commitment, deserialized);
    }
}

impl From<Commitment> for pb::NoteCommitment {
    fn from(nc: Commitment) -> Self {
        Self {
            inner: nc.0.to_bytes().to_vec(),
        }
    }
}

/// Error returned when a note commitment cannot be deserialized because it is not in range.
#[derive(thiserror::Error, Debug, Clone, Copy)]
#[error("Invalid note commitment")]
pub struct InvalidNoteCommitment;

impl TryFrom<pb::NoteCommitment> for Commitment {
    type Error = InvalidNoteCommitment;

    fn try_from(value: pb::NoteCommitment) -> Result<Self, Self::Error> {
        let bytes: [u8; 32] = value.inner[..]
            .try_into()
            .map_err(|_| InvalidNoteCommitment)?;

        let inner = Fq::from_bytes(bytes).map_err(|_| InvalidNoteCommitment)?;

        Ok(Commitment(inner))
    }
}

impl std::fmt::Display for Commitment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(&self.0.to_bytes()[..]))
    }
}

impl std::fmt::Debug for Commitment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "note::Commitment({})",
            hex::encode(&self.0.to_bytes()[..])
        ))
    }
}

impl From<Commitment> for [u8; 32] {
    fn from(commitment: Commitment) -> [u8; 32] {
        commitment.0.to_bytes()
    }
}

impl TryFrom<[u8; 32]> for Commitment {
    type Error = InvalidNoteCommitment;

    fn try_from(bytes: [u8; 32]) -> Result<Commitment, Self::Error> {
        let inner = Fq::from_bytes(bytes).map_err(|_| InvalidNoteCommitment)?;

        Ok(Commitment(inner))
    }
}

// TODO: remove? aside from sqlx is there a use case for non-proto conversion from byte slices?
impl TryFrom<&[u8]> for Commitment {
    type Error = InvalidNoteCommitment;

    fn try_from(slice: &[u8]) -> Result<Commitment, Self::Error> {
        let bytes: [u8; 32] = slice[..].try_into().map_err(|_| InvalidNoteCommitment)?;

        let inner = Fq::from_bytes(bytes).map_err(|_| InvalidNoteCommitment)?;

        Ok(Commitment(inner))
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use super::Commitment;

    // Arbitrary implementation for [`Commitment`]s.

    impl proptest::arbitrary::Arbitrary for Commitment {
        type Parameters = Vec<Commitment>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            CommitmentStrategy(args)
        }

        type Strategy = CommitmentStrategy;
    }

    /// A [`proptest`] [`Strategy`](proptest::strategy::Strategy) for generating [`Commitment`]s.
    #[derive(Clone, Debug, PartialEq, Eq, Default)]
    pub struct CommitmentStrategy(Vec<Commitment>);

    impl CommitmentStrategy {
        /// Create a new [`CommitmentStrategy`] that will generate arbitrary [`Commitment`]s.
        pub fn arbitrary() -> Self {
            Self::one_of(vec![])
        }

        /// Create a new [`CommitmentStrategy`] that will only produce the given [`Commitment`]s.
        ///
        /// If the given vector is empty, this will generate arbitrary commitments instead.
        pub fn one_of(commitments: Vec<Commitment>) -> Self {
            CommitmentStrategy(commitments)
        }
    }

    impl proptest::strategy::Strategy for CommitmentStrategy {
        type Tree = proptest::strategy::Just<Commitment>;

        type Value = Commitment;

        fn new_tree(
            &self,
            runner: &mut proptest::test_runner::TestRunner,
        ) -> proptest::strategy::NewTree<Self> {
            use proptest::prelude::{Rng, RngCore};
            let rng = runner.rng();
            if !self.0.is_empty() {
                Ok(proptest::strategy::Just(
                    *rng.sample(rand::distributions::Slice::new(&self.0).unwrap()),
                ))
            } else {
                let parts = [
                    rng.next_u64(),
                    rng.next_u64(),
                    rng.next_u64(),
                    rng.next_u64(),
                ];
                Ok(proptest::strategy::Just(Commitment(decaf377::Fq::new(
                    ark_ff::BigInteger256(parts),
                ))))
            }
        }
    }
}
