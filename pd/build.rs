use std::path::Path;

use anyhow::Context;
use vergen::{vergen, Config};

fn main() -> anyhow::Result<()> {
    vergen(Config::default()).unwrap();
    setup_testnet_config()?;
    Ok(())
}

// Set build-time environment variables to point to the latest testnet's config files.
fn setup_testnet_config() -> anyhow::Result<()> {
    // Get the path to the testnets directory, in a platform-agnostic manner
    let testnets_path = std::env::current_dir()
        .context("could not get current working directory")?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("could not get parent of current working directory"))?
        .join("testnets");

    // Get the most recent testnet name and its configuration directory
    let (latest_testnet_name, latest_testnet_dir) = latest_testnet(&testnets_path)?;

    // Output the name of the most recent testnet as a build-time environment variable
    println!("cargo:rustc-env=PD_LATEST_TESTNET_NAME={latest_testnet_name}");

    // For each association of environment variable to filename, set the full path to that file in
    // the environment variable, so that we can include its contents at build time
    for (env_var, filename) in [
        ("PD_LATEST_TESTNET_ALLOCATIONS", "allocations.csv"),
        ("PD_LATEST_TESTNET_VALIDATORS", "validators.json"),
    ] {
        let path = testnets_path.join(&latest_testnet_dir).join(filename);
        println!(
            "cargo:rustc-env={}={}",
            env_var,
            path.to_str()
                .ok_or_else(|| anyhow::anyhow!("invalid UTF-8 in path"))?
        );
    }

    Ok(())
}

// Scan through the testnets directory to find the latest one
fn latest_testnet(testnets_path: impl AsRef<Path>) -> anyhow::Result<(String, String)> {
    let mut testnets = Vec::new();
    for result in std::fs::read_dir(testnets_path.as_ref()).with_context(|| {
        format!(
            "could not read testnet directory {:?}",
            testnets_path.as_ref()
        )
    })? {
        let entry = result.context("error reading directory entry")?;
        if entry
            .file_type()
            .context("error checking filetype of directory entry")?
            .is_dir()
        {
            let path = entry.path();
            let dir_name = entry
                .file_name()
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("testnet path '{:?}' is invalid utf8", path))?
                .to_string();
            // Split the testnet directory name into (index, name), i.e. `001-valetudo`
            // becomes (1, "valetudo")
            let (index, name): (u64, _) = dir_name
                .split_once('-')
                .ok_or_else(|| {
                    anyhow::anyhow!("testnet path '{:?}' is not correctly formatted", path)
                })
                .and_then(|(index_str, name)| {
                    Ok((
                        index_str.parse().with_context(|| {
                            format!("could not parse testnet index as a number in path '{path:?}'")
                        })?,
                        name.to_string(),
                    ))
                })?;
            testnets.push((index, name, dir_name));
        }
    }

    // Compute the maximum index testnet in the testnets directory
    testnets
        .into_iter()
        .max_by_key(|(index, _, _)| *index)
        .map(|(_, name, dir_name)| ("penumbra-testnet-".to_string() + &name, dir_name))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no testnets found in directory {:?}",
                testnets_path.as_ref()
            )
        })
}
