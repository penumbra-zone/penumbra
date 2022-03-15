#![allow(unused_imports)]
use penumbra_tct::{Block, Commitment, Epoch, Eternity, Forget, Keep};

use ark_ff::PrimeField;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = Eternity::new();

    for (i, witness) in [1u64].into_iter().zip([Keep].into_iter().cycle()) {
        let fq = Commitment::from_le_bytes_mod_order(&i.to_le_bytes());
        tree.insert(witness, fq)?;
    }

    tree.insert_block(Block::new())?;

    println!("{tree:?}");

    for (i, witness) in [1u64].into_iter().zip([Keep].into_iter().cycle()) {
        let fq = Commitment::from_le_bytes_mod_order(&i.to_le_bytes());
        tree.insert(witness, fq)?;
    }

    tree.insert_epoch(Epoch::new())?;

    println!("{tree:?}");

    for (i, witness) in [1u64].into_iter().zip([Keep].into_iter().cycle()) {
        let fq = Commitment::from_le_bytes_mod_order(&i.to_le_bytes());
        tree.insert(witness, fq)?;
    }

    println!("{tree:?}");

    let root = tree.root();
    let proof = tree
        .witness(Commitment::from_le_bytes_mod_order(&1u64.to_le_bytes()))
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
