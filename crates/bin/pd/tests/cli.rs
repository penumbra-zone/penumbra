use assert_cmd::Command;
use tempfile::tempdir;

#[test]
/// Ensure that `pd testnet generate` passes with default settings.
/// Uses a tempdir for the state.
fn generate_network() -> anyhow::Result<()> {
    let tmpdir = tempdir()?;
    let mut gen_cmd = Command::cargo_bin("pd")?;
    gen_cmd.args([
        "testnet",
        "--testnet-dir",
        tmpdir.path().to_str().unwrap(),
        "generate",
    ]);
    gen_cmd.assert().success();
    Ok(())
}
