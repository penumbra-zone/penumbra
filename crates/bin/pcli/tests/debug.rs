use core::fmt::Debug;
use penumbra_proof_params::{SPEND_PROOF_PROVING_KEY, SPEND_PROOF_VERIFICATION_KEY};
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

fn print_to_file<T: Debug>(data: &T, filename: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    writeln!(file, "{:#?}", data)?;
    Ok(())
}

#[test]
fn spend_debug() {
    let _ = fs::remove_file("spend_proof.txt");

    let pk = &*SPEND_PROOF_PROVING_KEY;
    print_to_file(pk, "spend_proof.txt").expect("Failed to write proving key");

    let vk = &*SPEND_PROOF_VERIFICATION_KEY;
    print_to_file(vk, "spend_proof.txt").expect("Failed to write verification key");
}
