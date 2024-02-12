use decaf377::Fr;

pub fn to_bit(a: &[u8; 32], b: &[u8; 32], c: &[u8; 32]) -> u8 {
    blake2b_simd::Params::default()
        .personal(b"decaf377-fmd.bit")
        .to_state()
        .update(a)
        .update(b)
        .update(c)
        .finalize()
        .as_bytes()[0]
        & 1
}

pub fn to_scalar(point: &[u8; 32], n: u8, bits: &[u8]) -> Fr {
    // assert to avoid forcing callers to copy into another array
    assert_eq!(bits.len(), 3);

    let hash = blake2b_simd::Params::default()
        .personal(b"decaf377-fmd.bit")
        .to_state()
        .update(point)
        .update(&[n])
        .update(bits)
        .finalize();

    Fr::from_le_bytes_mod_order(hash.as_bytes())
}
