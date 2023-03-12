use std::path::PathBuf;
use std::{
    env, fs,
    io::{BufWriter, Result},
};

use ark_groth16::{ProvingKey, VerifyingKey};
use ark_serialize::CanonicalSerialize;
use decaf377::Bls12_377;
use penumbra_crypto::proofs::groth16::{
    OutputCircuit, ParameterSetup, ProvingKeyExt, SpendCircuit, SwapCircuit, SwapClaimCircuit,
    VerifyingKeyExt,
};

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

    // Generate the parameters for the current proofs and serialize them
    // to files in the target directory.
    let (spend_pk, spend_vk) = SpendCircuit::generate_test_parameters();
    write_params(&target_dir, "spend", &spend_pk, &spend_vk)?;
    let (output_pk, output_vk) = OutputCircuit::generate_test_parameters();
    write_params(&target_dir, "output", &output_pk, &output_vk)?;
    let (swap_pk, swap_vk) = SwapCircuit::generate_test_parameters();
    write_params(&target_dir, "swap", &swap_pk, &swap_vk)?;
    let (swapclaim_pk, swapclaim_vk) = SwapClaimCircuit::generate_test_parameters();
    write_params(&target_dir, "swapclaim", &swapclaim_pk, &swapclaim_vk)?;
    // NOTE: New proofs go here following the approach above.

    Ok(())
}

fn write_params(
    target_dir: &PathBuf,
    name: &str,
    pk: &ProvingKey<Bls12_377>,
    vk: &VerifyingKey<Bls12_377>,
) -> Result<()> {
    let pk_location = target_dir.join(format!("{}_pk.bin", name));
    let vk_location = target_dir.join(format!("{}_vk.param", name));
    let id_location = target_dir.join(format!("{}_id.rs", name));

    let pk_file = fs::File::create(&pk_location)?;
    let vk_file = fs::File::create(&vk_location)?;

    let pk_writer = BufWriter::new(pk_file);
    let vk_writer = BufWriter::new(vk_file);

    ProvingKey::serialize_unchecked(pk, pk_writer).expect("can serialize ProvingKey");
    VerifyingKey::serialize_unchecked(vk, vk_writer).expect("can serialize VerifyingKey");

    let pk_id = pk.debug_id();
    let vk_id = vk.debug_id();
    std::fs::write(
        id_location,
        format!(
            r#"
pub const PROVING_KEY_ID: &'static str = "{pk_id}";
pub const VERIFICATION_KEY_ID: &'static str = "{vk_id}";
"#,
        ),
    )?;

    Ok(())
}
