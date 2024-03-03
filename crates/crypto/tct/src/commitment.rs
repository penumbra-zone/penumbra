use decaf377::Fq;
use penumbra_proto::{penumbra::crypto::tct::v1 as pb, DomainType};

/// A commitment to a note or swap.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "pb::StateCommitment", try_from = "pb::StateCommitment")]
pub struct StateCommitment(pub Fq);

/// An error when decoding a commitment from a hex string.
#[derive(Clone, Debug, thiserror::Error)]
pub enum ParseCommitmentError {
    /// The string was not a hex string.
    #[error(transparent)]
    InvalidHex(#[from] hex::FromHexError),
    /// The bytes did not encode a valid commitment.
    #[error(transparent)]
    InvalidCommitment(#[from] InvalidStateCommitment),
}

impl StateCommitment {
    /// Parse a hex string as a [`Commitment`].
    pub fn parse_hex(str: &str) -> Result<StateCommitment, ParseCommitmentError> {
        let bytes = hex::decode(str)?;
        Ok(StateCommitment::try_from(&bytes[..])?)
    }
}

impl DomainType for StateCommitment {
    type Proto = pb::StateCommitment;
}

#[cfg(test)]
mod test_serde {
    use super::StateCommitment;

    #[test]
    fn roundtrip_json_zero() {
        let commitment = StateCommitment::try_from([0; 32]).unwrap();
        let bytes = serde_json::to_vec(&commitment).unwrap();
        println!("{bytes:?}");
        let deserialized: StateCommitment = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(commitment, deserialized);
    }

    /*
    Disabled; pbjson_build derived implementations don't play well with bincode,
    because of the issue described here: https://github.com/bincode-org/bincode/issues/276
    #[test]
    fn roundtrip_bincode_zero() {
        let commitment = Commitment::try_from([0; 32]).unwrap();
        let bytes = bincode::serialize(&commitment).unwrap();
        println!("{:?}", bytes);
        let deserialized: Commitment = bincode::deserialize(&bytes).unwrap();
        assert_eq!(commitment, deserialized);
    }
     */
}

impl From<StateCommitment> for pb::StateCommitment {
    fn from(nc: StateCommitment) -> Self {
        Self {
            inner: nc.0.to_bytes().to_vec(),
        }
    }
}

/// Error returned when a note commitment cannot be deserialized because it is not in range.
#[derive(thiserror::Error, Debug, Clone, Copy)]
#[error("Invalid note commitment")]
pub struct InvalidStateCommitment;

impl TryFrom<pb::StateCommitment> for StateCommitment {
    type Error = InvalidStateCommitment;

    fn try_from(value: pb::StateCommitment) -> Result<Self, Self::Error> {
        let bytes: [u8; 32] = value.inner[..]
            .try_into()
            .map_err(|_| InvalidStateCommitment)?;

        let inner = Fq::from_bytes_checked(&bytes).map_err(|_| InvalidStateCommitment)?;

        Ok(StateCommitment(inner))
    }
}

impl std::fmt::Display for StateCommitment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(&self.0.to_bytes()[..]))
    }
}

impl std::fmt::Debug for StateCommitment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "note::Commitment({})",
            hex::encode(&self.0.to_bytes()[..])
        ))
    }
}

impl From<StateCommitment> for [u8; 32] {
    fn from(commitment: StateCommitment) -> [u8; 32] {
        commitment.0.to_bytes()
    }
}

impl TryFrom<[u8; 32]> for StateCommitment {
    type Error = InvalidStateCommitment;

    fn try_from(bytes: [u8; 32]) -> Result<StateCommitment, Self::Error> {
        let inner = Fq::from_bytes_checked(&bytes).map_err(|_| InvalidStateCommitment)?;

        Ok(StateCommitment(inner))
    }
}

// TODO: remove? aside from sqlx is there a use case for non-proto conversion from byte slices?
impl TryFrom<&[u8]> for StateCommitment {
    type Error = InvalidStateCommitment;

    fn try_from(slice: &[u8]) -> Result<StateCommitment, Self::Error> {
        let bytes: [u8; 32] = slice[..].try_into().map_err(|_| InvalidStateCommitment)?;

        let inner = Fq::from_bytes_checked(&bytes).map_err(|_| InvalidStateCommitment)?;

        Ok(StateCommitment(inner))
    }
}

#[cfg(feature = "arbitrary")]
pub use arbitrary::FqStrategy;

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use decaf377::Fq;
    use proptest::strategy::Strategy;

    use super::StateCommitment;

    // Arbitrary implementation for [`Commitment`]s.
    impl proptest::arbitrary::Arbitrary for StateCommitment {
        type Parameters = Vec<StateCommitment>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            FqStrategy(args.into_iter().map(|commitment| commitment.0).collect())
                .prop_map(StateCommitment)
        }

        type Strategy = proptest::strategy::Map<FqStrategy, fn(Fq) -> StateCommitment>;
    }

    /// A [`proptest`] [`Strategy`](proptest::strategy::Strategy) for generating [`Fq`]s.
    #[derive(Clone, Debug, PartialEq, Eq, Default)]
    pub struct FqStrategy(Vec<Fq>);

    impl FqStrategy {
        /// Create a new [`FqStrategy`] that will generate arbitrary [`Commitment`]s.
        pub fn arbitrary() -> Self {
            Self::one_of(vec![])
        }

        /// Create a new [`FqStrategy`] that will only produce the given [`Fq`]s.
        ///
        /// If the given vector is empty, this will generate arbitrary [`Fq`]s instead.
        pub fn one_of(commitments: Vec<Fq>) -> Self {
            FqStrategy(commitments)
        }
    }

    impl proptest::strategy::Strategy for FqStrategy {
        type Tree = proptest::strategy::Filter<proptest::strategy::Just<Fq>, fn(&Fq) -> bool>;

        type Value = Fq;

        fn new_tree(
            &self,
            runner: &mut proptest::test_runner::TestRunner,
        ) -> proptest::strategy::NewTree<Self> {
            use proptest::prelude::{Rng, RngCore};
            let rng = runner.rng();
            Ok(if !self.0.is_empty() {
                proptest::strategy::Just(
                    *rng.sample(rand::distributions::Slice::new(&self.0).expect("empty vector")),
                )
            } else {
                let mut bytes = [0u8; 32];
                rng.fill_bytes(&mut bytes);
                proptest::strategy::Just(decaf377::Fq::from_le_bytes_mod_order(&bytes))
            }
            .prop_filter("impossible", |_| true))
        }
    }
}
