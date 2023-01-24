use std::path::PathBuf;
use std::{
    env, fs,
    io::{BufWriter, Result},
};

use ark_groth16::{ProvingKey, VerifyingKey};
use ark_serialize::CanonicalSerialize;
use decaf377::Bls12_377;
use penumbra_crypto::proofs::groth16::{OutputCircuit, ParameterSetup, SpendCircuit};

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("{}", root.display());
    let target_dir = root
        .join("..")
        .join("..")
        .join("proof-params")
        .join("src")
        .join("gen");
    println!("{}", target_dir.display());

    // Generate the parameters for the current proofs.
    let (spend_pk, spend_vk) = SpendCircuit::generate_test_parameters();
    let (output_pk, output_vk) = OutputCircuit::generate_test_parameters();

    // Serialize the parameters to files in the target directory.
    write_params(&target_dir, "spend", &spend_pk, &spend_vk)?;
    write_params(&target_dir, "output", &output_pk, &output_vk)?;

    Ok(())
}

fn write_params(
    target_dir: &PathBuf,
    name: &str,
    pk: &ProvingKey<Bls12_377>,
    vk: &VerifyingKey<Bls12_377>,
) -> Result<()> {
    let pk_location = target_dir.join(format!("{}_pk.bin", name));
    let vk_location = target_dir.join(format!("{}_vk.bin", name));

    let pk_file = fs::File::create(&pk_location)?;
    let vk_file = fs::File::create(&vk_location)?;

    let pk_writer = BufWriter::new(pk_file);
    let vk_writer = BufWriter::new(vk_file);

    ProvingKey::serialize(pk, pk_writer).expect("can serialize ProvingKey");
    VerifyingKey::serialize(vk, vk_writer).expect("can serialize VerifyingKey");

    Ok(())
}
