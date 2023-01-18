//! Contains an implementation of a 32-byte big-ending fixed point encoding of fraction values consisting
//! of a u128 integer portion and u128 decimal portion.
use core::fmt::Debug;
use std::fmt::Display;

pub struct FixedEncoding([u8; 32]);

impl FixedEncoding {
    /// Constructs a new fixed point encoding based on an integer portion and a decimal portion.
    pub fn new(integer: u128, decimal: u128) -> Self {
        let mut result = [0u8; 32];
        result[..16].copy_from_slice(&integer.to_be_bytes());
        result[16..].copy_from_slice(&decimal.to_be_bytes());
        Self(result)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }

    pub fn to_parts(&self) -> (u128, u128) {
        let integer = u128::from_be_bytes(self.0[..16].try_into().unwrap());
        let decimal = u128::from_be_bytes(self.0[16..].try_into().unwrap());
        (integer, decimal)
    }
}

impl PartialEq for FixedEncoding {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes().eq(other.as_bytes())
    }
}

impl Eq for FixedEncoding {}

impl PartialOrd for FixedEncoding {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_bytes().partial_cmp(other.as_bytes())
    }
}

impl Ord for FixedEncoding {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl Debug for FixedEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.as_bytes(),))
    }
}

impl Display for FixedEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "N: {} D: {}",
            self.to_parts().0,
            self.to_parts().1
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let test_numbers: Vec<(u128, u128)> = vec![
            // 0.0
            (0, 0),
            // 10 * 10^-39
            (0, 1),
            // 10 * 10^-38
            (0, 10),
            // 1.0
            (1, 0),
            // 1 + 10*10^-39
            (1, 1),
            // 1 + 10*10^-38
            (1, 10),
            // 10 + 10*10^-38
            (10, 10),
        ];

        for number in test_numbers {
            let (i, d) = number;
            let encoded = FixedEncoding::new(i, d);
            let (i2, d2) = encoded.to_parts();
            assert_eq!(i, i2);
            assert_eq!(d, d2);
        }
    }

    #[test]
    fn test_ordering() {
        // The lexicographic ordering of the fixed encoding should be numeric ordering.

        // A set of ordered test fractions.
        let test_numbers: Vec<(u128, u128)> = vec![
            // 0.0
            (0, 0),
            // 10 * 10^-39
            (0, 1),
            // 10 * 10^-38
            (0, 10),
            // 1.0
            (1, 0),
            // 1 + 10*10^-39
            (1, 1),
            // 1 + 10*10^-38
            (1, 10),
            // 10 + 10*10^-38
            (10, 10),
        ];

        let mut numbers_fp = test_numbers
            .iter()
            .map(|(n, d)| FixedEncoding::new(*n, *d))
            .collect::<Vec<_>>();

        // If we lexicographically sort the bytes of the fixed encodings, we should get the same numeric ordering as the
        // original test numbers.
        numbers_fp.sort();

        for (i, number) in numbers_fp.iter().enumerate() {
            let (integer, decimal) = number.to_parts();
            assert_eq!(integer, test_numbers[i].0);
            assert_eq!(decimal, test_numbers[i].1);
        }
    }
}
