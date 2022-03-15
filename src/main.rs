#![allow(unused_imports)]
use penumbra_tct::{Block, Commitment, Epoch, Eternity, Forget, Keep};

use ark_ff::PrimeField;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = Eternity::new();

    for (i, witness) in (50u64..75).zip([Keep, Forget, Forget, Forget, Keep].into_iter().cycle()) {
        let fq = Commitment::from_le_bytes_mod_order(&i.to_le_bytes());
        tree.insert(witness, fq).unwrap();
    }

    println!("{tree:?}");

    let root = tree.root();
    let proof = tree
        .witness(Commitment::from_le_bytes_mod_order(&50u64.to_le_bytes()))
        .unwrap();

    assert!(proof.verify(&root).is_ok());
    assert!(tree
        .witness(Commitment::from_le_bytes_mod_order(&51u64.to_le_bytes()))
        .is_none());
    assert!(tree
        .witness(Commitment::from_le_bytes_mod_order(&5000u64.to_le_bytes()))
        .is_none());

    Ok(())
}
