#![allow(unused_imports)]
use penumbra_tct::{Block, Epoch, Eternity, Fq, Hash, Insert};

use ark_ff::PrimeField;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = Eternity::new();

    for (i, keep) in (0u64..1000).zip([true].into_iter().cycle()) {
        let fq = Fq::from_le_bytes_mod_order(&i.to_le_bytes());
        if keep {
            tree.insert_item(Insert::Keep(fq)).unwrap();
        } else {
            tree.insert_item(Insert::Hash(Hash::of(fq))).unwrap();
        }
    }

    // println!("{tree:?}");

    let root = tree.root();
    let proof = tree
        .witness(Fq::from_le_bytes_mod_order(&0u64.to_le_bytes()))
        .unwrap();

    assert!(proof.verify(root).is_ok());
    assert!(tree
        .witness(Fq::from_le_bytes_mod_order(&5000u64.to_le_bytes()))
        .is_none());

    Ok(())
}
