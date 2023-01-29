use decaf377::FieldExt;
use penumbra_proto::{core::crypto::v1alpha1 as pb, Protobuf};
use poseidon377::Fq;

/// A commitment to a note or swap.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "pb::StateCommitment", try_from = "pb::StateCommitment")]
pub struct Commitment(pub Fq);

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

impl Commitment {
    /// Parse a hex string as a [`Commitment`].
    pub fn parse_hex(str: &str) -> Result<Commitment, ParseCommitmentError> {
        let bytes = hex::decode(str)?;
        Ok(Commitment::try_from(&bytes[..])?)
    }
}

impl Protobuf for Commitment {
    type Proto = pb::StateCommitment;
}

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

impl From<Commitment> for pb::StateCommitment {
    fn from(nc: Commitment) -> Self {
        Self {
            inner: nc.0.to_bytes().to_vec(),
        }
    }
}

/// Error returned when a note commitment cannot be deserialized because it is not in range.
#[derive(thiserror::Error, Debug, Clone, Copy)]
#[error("Invalid note commitment")]
pub struct InvalidStateCommitment;

impl TryFrom<pb::StateCommitment> for Commitment {
    type Error = InvalidStateCommitment;

    fn try_from(value: pb::StateCommitment) -> Result<Self, Self::Error> {
        let bytes: [u8; 32] = value.inner[..]
            .try_into()
            .map_err(|_| InvalidStateCommitment)?;

        let inner = Fq::from_bytes(bytes).map_err(|_| InvalidStateCommitment)?;

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
    type Error = InvalidStateCommitment;

    fn try_from(bytes: [u8; 32]) -> Result<Commitment, Self::Error> {
        let inner = Fq::from_bytes(bytes).map_err(|_| InvalidStateCommitment)?;

        Ok(Commitment(inner))
    }
}

// TODO: remove? aside from sqlx is there a use case for non-proto conversion from byte slices?
impl TryFrom<&[u8]> for Commitment {
    type Error = InvalidStateCommitment;

    fn try_from(slice: &[u8]) -> Result<Commitment, Self::Error> {
        let bytes: [u8; 32] = slice[..].try_into().map_err(|_| InvalidStateCommitment)?;

        let inner = Fq::from_bytes(bytes).map_err(|_| InvalidStateCommitment)?;

        Ok(Commitment(inner))
    }
}

#[cfg(feature = "arbitrary")]
pub use arbitrary::FqStrategy;

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use ark_ed_on_bls12_377::{Fq, FqParameters};
    use ark_ff::FpParameters;
    use proptest::strategy::Strategy;

    use super::Commitment;

    // Arbitrary implementation for [`Commitment`]s.
    impl proptest::arbitrary::Arbitrary for Commitment {
        type Parameters = Vec<Commitment>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            FqStrategy(args.into_iter().map(|commitment| commitment.0).collect())
                .prop_map(Commitment)
        }

        type Strategy = proptest::strategy::Map<FqStrategy, fn(Fq) -> Commitment>;
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
                    *rng.sample(rand::distributions::Slice::new(&self.0).unwrap()),
                )
            } else {
                let parts = [
                    rng.next_u64(),
                    rng.next_u64(),
                    rng.next_u64(),
                    rng.next_u64(),
                ];
                proptest::strategy::Just(decaf377::Fq::new(ark_ff::BigInteger256(parts)))
            }
            .prop_filter("bigger than modulus", |fq| fq.0 < FqParameters::MODULUS))
        }
    }
}
