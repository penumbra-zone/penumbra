#![allow(unused_imports)]
use penumbra_tct::{Block, Commitment, Epoch, Eternity, Forget, Keep};

use ark_ff::PrimeField;
use decaf377::Fq;

fn commit(n: u64) -> Commitment {
    Commitment::from(Fq::from_le_bytes_mod_order(&n.to_le_bytes()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = Eternity::new();

    for (i, witness) in [1u64].into_iter().zip([Keep].into_iter().cycle()) {
        let c = commit(i);
        tree.insert(witness, c)?;
    }

    tree.insert_block(Block::new())?;

    println!("{tree:?}");

    for (i, witness) in [1u64].into_iter().zip([Keep].into_iter().cycle()) {
        let c = commit(i);
        tree.insert(witness, c)?;
    }

    tree.insert_epoch(Epoch::new())?;

    println!("{tree:?}");

    for (i, witness) in [1u64].into_iter().zip([Keep].into_iter().cycle()) {
        let c = commit(i);
        tree.insert(witness, c)?;
    }

    println!("{tree:?}");

    let root = tree.root();
    let proof = tree.witness(commit(1)).unwrap();

    assert!(proof.verify(&root).is_ok());
    assert!(tree.witness(commit(2)).is_none());
    assert!(tree.witness(commit(5000)).is_none());

    let forgotten = tree.forget(commit(1));
    assert!(forgotten);

    Ok(())
}
