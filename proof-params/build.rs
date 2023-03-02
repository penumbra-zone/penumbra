fn main() {
    let proving_parameter_files = [
        "src/gen/output_pk.bin",
        "src/gen/spend_pk.bin",
        "src/gen/swap_pk.bin",
    ];
    let verification_parameter_files = [
        "src/gen/output_vk.param",
        "src/gen/spend_vk.param",
        "src/gen/swap_vk.param",
    ];
    for file in proving_parameter_files
        .into_iter()
        .chain(verification_parameter_files)
    {
        println!("cargo:rerun-if-changed={file}");
    }
}
