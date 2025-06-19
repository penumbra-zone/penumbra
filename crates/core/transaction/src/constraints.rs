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

pub fn generate_and_serialize_circuit(circuit: ActionCircuit) -> Result<(Vec<u8>, usize)> {
    //  Generate constraints only
    let cs = build_constraint_system(circuit)?;

    // Serialize existing constraint system
    let (witness_bytes, public_inputs) = serialize_witness(&cs)?;

    Ok((witness_bytes, public_inputs))
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

/// Convenience method for constraint synthesis.
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

/// Convenience method for serializing witness and R1CS matrices.
pub fn serialize_witness(cs: &ConstraintSystemRef<Fq>) -> Result<(Vec<u8>, usize)> {
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

    // serialized witness data as byte vector
    Ok((witness_bytes, public_values.len()))
}
