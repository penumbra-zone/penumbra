use anyhow::Result;
use ark_ff::BigInteger;
use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef};
use ark_serialize::CanonicalSerialize;
use ark_serialize::Compress;
use byteorder::LittleEndian;
use byteorder::WriteBytesExt;
use decaf377::Fq;
use std::io::Write;

use crate::action::ActionCircuit;

pub fn generate_and_serialize_circuit(circuit: ActionCircuit) -> Result<(Vec<u8>, Vec<u8>)> {
    //  Generate constraints only
    let cs = build_constraint_system(circuit)?;

    // Serialize existing constraint system
    let (witness_bytes, matrices_bytes) = serialize_witness_and_matrices(&cs)?;

    Ok((witness_bytes, matrices_bytes))
}

fn generate_circuit_constraints<C: ConstraintSynthesizer<Fq> + Clone + std::fmt::Debug>(
    cs: ConstraintSystemRef<Fq>,
    circuit: C,
) {
    circuit
        .clone()
        .generate_constraints(cs)
        .expect("can generate constraints");
}

pub fn build_constraint_system(circuit: ActionCircuit) -> Result<ConstraintSystemRef<Fq>> {
    let cs: ConstraintSystemRef<_> = ConstraintSystem::new_ref();

    match circuit {
        ActionCircuit::Spend(circuit) => {
            generate_circuit_constraints(cs.clone(), circuit);
        }
        ActionCircuit::Output(circuit) => {
            generate_circuit_constraints(cs.clone(), circuit);
        }
        ActionCircuit::Swap(circuit) => {
            generate_circuit_constraints(cs.clone(), circuit);
        }
        ActionCircuit::SwapClaim(circuit) => {
            generate_circuit_constraints(cs.clone(), circuit);
        }
        ActionCircuit::DelegatorVote(circuit) => {
            generate_circuit_constraints(cs.clone(), circuit);
        }
    }

    cs.finalize();
    if !cs.is_satisfied()? {
        anyhow::bail!("constraints are not satisfied");
    }

    Ok(cs)
}

pub fn serialize_witness_and_matrices(cs: &ConstraintSystemRef<Fq>) -> Result<(Vec<u8>, Vec<u8>)> {
    let witness_values = &cs.borrow().expect("can borrow").witness_assignment;
    let public_values = &cs.borrow().expect("can borrow").instance_assignment;

    let mut witness_bytes = Vec::new();
    let mut witness_cursor = std::io::Cursor::new(&mut witness_bytes);

    witness_cursor.write_all(b"wtns")?;
    witness_cursor.write_u32::<LittleEndian>(2)?;
    witness_cursor.write_u32::<LittleEndian>(2)?;

    witness_cursor.write_u32::<LittleEndian>(0)?;
    witness_cursor.write_u64::<LittleEndian>(0)?;

    let modulus = Fq::MODULUS.to_bytes_le();
    witness_cursor.write_u32::<LittleEndian>(modulus.len() as u32)?;
    witness_cursor.write_all(&modulus)?;

    witness_cursor
        .write_u32::<LittleEndian>((witness_values.len() + public_values.len()) as u32)?;

    witness_cursor.write_u32::<LittleEndian>(0)?;
    witness_cursor.write_u64::<LittleEndian>(0)?;

    for v in public_values.iter().chain(witness_values) {
        v.serialize_with_mode(&mut witness_cursor, Compress::Yes)?;
    }

    let mut matrices_bytes = Vec::new();
    let mut matrices_cursor = std::io::Cursor::new(&mut matrices_bytes);

    let matrices = cs.to_matrices().expect("can gen matrices");
    (matrices.a, matrices.b, matrices.c).serialize_uncompressed(&mut matrices_cursor)?;

    // return serialized data as byte vectors.
    Ok((witness_bytes, matrices_bytes))
}
