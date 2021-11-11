use ark_ff::PrimeField;

pub fn expand(label: &'static [u8; 16], key: &[u8], input: &[u8]) -> blake2b_simd::Hash {
    blake2b_simd::Params::new()
        .personal(label)
        .key(key)
        .hash(input)
}

pub fn expand_ff<F: PrimeField>(label: &'static [u8; 16], key: &[u8], input: &[u8]) -> F {
    F::from_le_bytes_mod_order(expand(label, key, input).as_bytes())
}
