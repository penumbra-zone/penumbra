//! Packed representations of directions through a depth-first traversal, suitable for dense,
//! non-incremental serialization.

use ark_ed_on_bls12_377::FqParameters;
use ark_ff::{fields::FpParameters, BigInteger, BigInteger256};

use super::*;

// TODO: this needs to be reworked to be many-to-many variable-length in order to support the use
// case of omitting (some) internal node hashes to compact space

/// A packed representation of `(Directive, Fq)` which takes up the same space (32 bytes) as an
/// [`Fq`].
///
/// This is accomplished by using the 3 highest bits to encode the [`Direction`], because the
/// modulus of [`Fq`] is low enough that these bits are never used to represent [`Fq`]s.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instruction {
    bytes: [u8; 32],
}

/// The mask for getting the tag bits out of the highest byte.
const TAG_MASK: u8 = 0b111_00000;

// Tags for each of the variants of [`Direction`].
const DOWN_0: u8 = 0b001_00000;
const DOWN_1: u8 = 0b010_00000;
const DOWN_2: u8 = 0b011_00000;
const DOWN_3: u8 = 0b100_00000;
const DOWN_4: u8 = 0b101_00000;
const RIGHT_OR_UP: u8 = 0b110_00000;

impl Instruction {
    /// Pack a [`Direction`] and an [`Fq`] into the space of a single [`Fq`].
    pub fn pack(direction: super::Instruction) -> Instruction {
        use {super::Instruction::*, Size::*};

        let mut bytes: [u8; 32] = element.0.to_bytes_le().try_into().unwrap();

        // Compute a bit mask to apply to the highest 3 bits of the highest byte
        let tag = match direction {
            Node(None) => DOWN_0,
            Node(Some(One)) => DOWN_1,
            Node(Some(Two)) => DOWN_2,
            Node(Some(Three)) => DOWN_3,
            Node(Some(Four)) => DOWN_4,
            Leaf => RIGHT_OR_UP,
        };

        // Apply the tag to the highest byte
        *bytes.last_mut().unwrap() |= tag;

        Self { bytes }
    }

    /// Unpack a [`PackedDirection`] into a [`Direction`] and an [`Fq`].
    pub fn unpack(self) -> (Instruction, Fq) {
        (self.direction(), self.element())
    }

    /// Get the [`Direction`] of this packed pair.
    pub fn direction(&self) -> Instruction {
        use {self::Instruction::*, Size::*};

        // Examine the highest three bits of the last byte to determine the direction
        match self.bytes.last().unwrap() & TAG_MASK {
            // The 0 tag is invalid so that if we mess up and interpret an ordinary `Fq` as a
            // `PackedDirection`, it will be immediately obvious, since all `Fq` have 0b000...
            // leading bits
            0b000_00000 => unreachable!("no tag for packed direction"),
            DOWN_0 => Down(None),
            DOWN_1 => Down(Some(One)),
            DOWN_2 => Down(Some(Two)),
            DOWN_3 => Down(Some(Three)),
            DOWN_4 => Down(Some(Four)),
            RIGHT_OR_UP => RightOrUp,
            0b111_00000 => unreachable!("nonexistent tag for packed direction"),
            _ => unreachable!("`x & 0b111_00000` never has 1 bits after the first 3 bits"),
        }
    }

    /// Get the [`Fq`] of this packed pair.
    pub fn element(&self) -> Fq {
        let mut bytes = self.bytes;

        // Mask out the tag bits (they are always zero for `Fq` because the modulus of `Fq` is 253)
        *bytes.last_mut().unwrap() &= !TAG_MASK;

        let mut integer = BigInteger256::from(0);
        integer
            .read_le(&mut std::io::Cursor::new(bytes))
            .expect("there are no read errors from arrays of correct size");

        Fq::new(integer)
    }
}

impl AsRef<[u8]> for Instruction {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl AsRef<[u8; 32]> for Instruction {
    fn as_ref(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl From<(Instruction, Fq)> for Instruction {
    fn from((direction, element): (Instruction, Fq)) -> Self {
        Self::pack(direction, element)
    }
}

impl From<Instruction> for (Instruction, Fq) {
    fn from(packed: Instruction) -> Self {
        packed.unpack()
    }
}

/// An error when parsing a [`PackedDirection`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Error)]
pub enum ParsePackedInstructionError {
    /// The tag could not be parsed as a [`Direction`].
    #[error("invalid tag on packed direction/element pair: {0:?}")]
    InvalidTag([u8; 32]),
    /// The element was out of range for an [`Fq`].
    #[error("element out of range in packed direction/element pair: {0:?}")]
    InvalidElement([u8; 32]),
}

impl TryFrom<[u8; 32]> for Instruction {
    type Error = ParsePackedInstructionError;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        // Validate the tag
        match bytes.last().unwrap() & TAG_MASK {
            // The 0 tag is invalid so that if we mess up and interpret an ordinary `Fq` as a
            // `PackedDirection`, it will be immediately obvious, since all `Fq` have 0b000...
            // leading bits
            DOWN_0 | DOWN_1 | DOWN_2 | DOWN_3 | DOWN_4 | RIGHT_OR_UP => {}
            0b000_00000 | 0b111_00000 => {
                return Err(ParsePackedInstructionError::InvalidTag(bytes))
            }
            _ => unreachable!("`x & 0b111_00000` never has 1 bits after the first 3 bits"),
        }

        // Remove the tag to validate the `Fq` is in bounds
        let mut untagged_bytes = bytes;
        *untagged_bytes.last_mut().unwrap() &= !TAG_MASK;

        // Parse the bytes as a little-endian big integer and check that they're less than the
        // modulus of `Fq`
        let mut integer = BigInteger256::from(0);
        integer
            .read_le(&mut std::io::Cursor::new(untagged_bytes))
            .unwrap();
        if integer >= FqParameters::MODULUS {
            return Err(ParsePackedInstructionError::InvalidElement(bytes));
        }

        // If both checks are ok, return the packed bytes
        Ok(Self { bytes })
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use super::*;

    /// Compute `Fq` modulus minus one.
    fn max_fq_value() -> Fq {
        let mut max_val = FqParameters::MODULUS;
        let borrow = max_val.sub_noborrow(&BigInteger256::from(1));
        assert!(
            !borrow,
            "large prime numbers are odd, so subtraction doesn't borrow"
        );
        Fq::new(max_val)
    }

    /// Check that the maximum representable `Fq` value doesn't use any of the tag bits, but that it
    /// does use some of the bits in the rest of that byte (i.e. it's little-byte-endian,
    /// big-bit-endian).
    #[test]
    fn tag_bits_unused() {
        assert_eq!(
            max_fq_value().0.to_bytes_le().last().unwrap() & 0b111_00000,
            0,
            "expecting tag bits to be unused"
        );
        assert_ne!(
            *max_fq_value().0.to_bytes_le().last().unwrap(),
            0,
            "expecting at least some bits of the highest byte to be used"
        );
        panic!("{}", max_fq_value().0)
    }

    proptest! {
        #[test]
        fn roundtrip_pack_unpack(
            direction in prop::arbitrary::any::<Instruction>(),
            fq in
                proptest::array::uniform32(0..u8::MAX)
                    .prop_filter_map("bigger than modulus", |bytes| {
                        let mut integer = BigInteger256::from(0);
                        integer.read_le(&mut std::io::Cursor::new(bytes)).unwrap();
                        if integer < FqParameters::MODULUS {
                            Some(Fq::new(integer))
                        } else {
                            None
                        }
                    })
        ) {
            let packed = Instruction::pack(direction, fq);
            let (direction_out, fq_out) = packed.unpack();
            assert_eq!(direction, direction_out);
            assert_eq!(fq, fq_out);
        }

        #[test]
        fn roundtrip_unpack_pack(
            packed in
                proptest::array::uniform32(0..u8::MAX)
                    .prop_filter_map("bigger than modulus", |bytes| {
                        Instruction::try_from(bytes).ok()
                    })
        ) {
            let (direction, fq) = packed.unpack();
            let packed_again = Instruction::pack(direction, fq);
            assert_eq!(packed, packed_again);
        }
    }
}
