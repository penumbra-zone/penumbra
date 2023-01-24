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
}
