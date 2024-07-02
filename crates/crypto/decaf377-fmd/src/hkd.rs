use decaf377::Fr;

#[allow(non_snake_case)]
pub fn derive_public(
    root_pub: &decaf377::Element,
    root_pub_enc: &decaf377::Encoding,
    index: u8,
) -> decaf377::Element {
    let hash = blake2b_simd::Params::default()
        .personal(b"decaf377-fmd.hkd")
        .to_state()
        .update(&root_pub_enc.0)
        .update(&[index])
        .finalize();
    let x = Fr::from_le_bytes_mod_order(hash.as_bytes());
    let X = x * decaf377::Element::GENERATOR;

    root_pub + X
}

pub fn derive_private(root_priv: &Fr, root_pub_enc: &decaf377::Encoding, index: u8) -> Fr {
    let hash = blake2b_simd::Params::default()
        .personal(b"decaf377-fmd.hkd")
        .to_state()
        .update(&root_pub_enc.0)
        .update(&[index])
        .finalize();
    let x = Fr::from_le_bytes_mod_order(hash.as_bytes());

    *root_priv + x
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn fr_strategy() -> BoxedStrategy<decaf377::Fr> {
        any::<[u8; 32]>()
            .prop_map(|bytes| decaf377::Fr::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    proptest! {
        #[test]
        fn public_private_derivation_match(root_priv in fr_strategy()) {
            let root_pub = root_priv * decaf377::Element::GENERATOR;
            let root_pub_enc = root_pub.vartime_compress();
            for i in 0..16u8 {
                let child_pub = derive_public(&root_pub, &root_pub_enc, i);
                let child_priv = derive_private(&root_priv, &root_pub_enc, i);
                let child_pub_from_priv = child_priv * decaf377::Element::GENERATOR;
                assert_eq!(child_pub, child_pub_from_priv);
            }
        }
    }
}
