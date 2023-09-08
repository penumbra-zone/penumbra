use std::path::PathBuf;
use std::{
    env, fs,
    io::{BufWriter, Result},
};

use ark_groth16::{ProvingKey, VerifyingKey};
use ark_serialize::CanonicalSerialize;
use decaf377::Bls12_377;
use penumbra_dex::{swap::proof::SwapCircuit, swap_claim::proof::SwapClaimCircuit};
use penumbra_governance::DelegatorVoteCircuit;
use penumbra_proof_params::{generate_test_parameters, VerifyingKeyExt, ProvingKeyExt};
use penumbra_shielded_pool::{NullifierDerivationCircuit, OutputCircuit, SpendCircuit};
use penumbra_stake::UndelegateClaimCircuit;
use rand_core::OsRng;

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("{}", root.display());
    let target_dir = root
        .join("..")
        .join("..")
        .join("crates")
        .join("crypto")
        .join("proof-params")
        .join("src")
        .join("gen");
    println!("{}", target_dir.display());

    // Generate the parameters for the current proofs and serialize them
    // to files in the target directory.
    let (spend_pk, spend_vk) = generate_test_parameters::<SpendCircuit>(&mut OsRng);
    write_params(&target_dir, "spend", &spend_pk, &spend_vk)?;
    let (output_pk, output_vk) = generate_test_parameters::<OutputCircuit>(&mut OsRng);
    write_params(&target_dir, "output", &output_pk, &output_vk)?;
    let (swap_pk, swap_vk) = generate_test_parameters::<SwapCircuit>(&mut OsRng);
    write_params(&target_dir, "swap", &swap_pk, &swap_vk)?;
    let (swapclaim_pk, swapclaim_vk) = generate_test_parameters::<SwapClaimCircuit>(&mut OsRng);
    write_params(&target_dir, "swapclaim", &swapclaim_pk, &swapclaim_vk)?;
    let (undelegateclaim_pk, undelegateclaim_vk) =
        generate_test_parameters::<UndelegateClaimCircuit>(&mut OsRng);
    write_params(
        &target_dir,
        "undelegateclaim",
        &undelegateclaim_pk,
        &undelegateclaim_vk,
    )?;
    let (delegator_vote_pk, delegator_vote_vk) =
        generate_test_parameters::<DelegatorVoteCircuit>(&mut OsRng);
    write_params(
        &target_dir,
        "delegator_vote",
        &delegator_vote_pk,
        &delegator_vote_vk,
    )?;
    let (nullifier_derivation_pk, nullifier_derivation_vk) =
        generate_test_parameters::<NullifierDerivationCircuit>(&mut OsRng);
    write_params(
        &target_dir,
        "nullifier_derivation",
        &nullifier_derivation_pk,
        &nullifier_derivation_vk,
    )?;
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

    ProvingKey::serialize_uncompressed(pk, pk_writer).expect("can serialize ProvingKey");
    VerifyingKey::serialize_uncompressed(vk, vk_writer).expect("can serialize VerifyingKey");

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
