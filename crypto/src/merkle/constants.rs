use ark_ff::PrimeField;
use decaf377::Fq;
use once_cell::sync::Lazy;

pub const ARITY: usize = 4;

// Since our tree arity is higher, to match the number of note commitments the Zcash sapling Merkle
// tree can hold we need only a depth of 16. This constant may change in the future after MVP1.
pub const MERKLE_DEPTH: usize = 16;

/// The domain separator used to hash items into the Merkle tree.
pub static MERKLE_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.merkle.tree").as_bytes())
});

/// The padding value used when hashing items into the Merkle tree.
pub static MERKLE_PADDING: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.merkle.padding").as_bytes())
});
