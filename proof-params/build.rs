use std::{env, path::PathBuf};

use penumbra_crypto::proofs::groth16::{OutputCircuit, ParameterSetup, SpendCircuit};

/// Each time you wish to update the parameters, you must:
/// 1. Run this build script.
/// 2. Increment the version number of the `penumbra-proof-params` crate.
/// 3. Commit the new parameters to the `penumbra-proof-params` crate.
fn main() {
    // We use the default `OUT_DIR` set by Cargo when a build script exists.
    let output_location: PathBuf =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environmental variable must be set"))
            .join("params.rs");

    // Generate the parameters for the current Spend and Output proofs.
    let (spend_pk, spend_vk) = SpendCircuit::generate_test_parameters();
    let (output_pk, output_vk) = OutputCircuit::generate_test_parameters();

    // TODO: Serialize the parameters to the file.
}
