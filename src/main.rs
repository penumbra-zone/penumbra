#![allow(unused_imports)]
use penumbra_tct::{Block, Epoch, Eternity, Fq, Hash, Insert, PrimeField};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = Eternity::new();
    for (i, keep) in (0usize..1000).zip([true].into_iter().cycle()) {
        let fq = Fq::from_le_bytes_mod_order(&i.to_le_bytes());
        if keep {
            tree.insert_item(Insert::Keep(fq)).unwrap();
        } else {
            tree.insert_item(Insert::Hash(Hash::of(fq))).unwrap();
        }
    }
    let root = tree.hash();
    let proof = tree.witness(0).unwrap();
    assert!(proof.verify(root).is_ok());
    assert!(tree.witness(usize::MAX).is_none());
    Ok(())
}
