fn main() {
    let proof_parameter_files = [
        "src/gen/output_pk.bin",
        "src/gen/output_vk.bin",
        "src/gen/spend_pk.bin",
        "src/gen/spend_vk.bin",
    ];
    for file in proof_parameter_files {
        println!("cargo:rerun-if-changed={file}");
    }

    // If the system where we are compiling `penumbra-proof-params` does not
    // have Git LFS installed, then the files will exist but they will be tiny
    // pointers. We want to detect this and panic if so, alerting the user
    // that they should go and install Git LFS.
    for file in proof_parameter_files {
        let metadata = std::fs::metadata(file).expect("proof parameter file exists");
        if metadata.len() < 500 {
            panic!(
                "proof parameter file {} is too small; did you install Git LFS?",
                file
            );
        }
    }
}
