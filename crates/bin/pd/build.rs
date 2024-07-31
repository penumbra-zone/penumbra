use std::io::Read;
use std::path::Path;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    check_frontend_asset_zipfiles()?;
    setup_testnet_config()?;
    Ok(())
}

// Check that the zip files for bundled frontend code are functional.
// If git-lfs is not configured on the build host, the zip files will
// be plaintext lfs pointer files.
fn check_frontend_asset_zipfiles() -> anyhow::Result<()> {
    // Declare a minimum filesize, below which we'll assume the zip file is
    // actually a git-lfs pointer.
    const MINIMUM_FILESIZE_BYTES: usize = 500;
    // Build paths to the zip files in the local build env.
    let zipfiles = vec![
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../assets/minifront.zip"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../assets/node-status.zip"),
    ];
    for zipfile in zipfiles {
        let mut bytes = Vec::new();
        let f = std::fs::File::open(&zipfile).context(format!(
            "failed to open zip file of frontend code: {}",
            &zipfile.display()
        ))?;
        let mut reader = std::io::BufReader::new(f);
        reader.read_to_end(&mut bytes).context(format!(
            "failed to read zip file of frontend code: {}",
            zipfile.display()
        ))?;
        if bytes.len() < MINIMUM_FILESIZE_BYTES {
            anyhow::bail!(
                format!(
                    "asset zip file {} is smaller than {} bytes; install git-lfs, run 'git lfs pull', and retry the build",
                    zipfile.display(),
                    MINIMUM_FILESIZE_BYTES
                )
                );
        }
    }
    Ok(())
}

// Set build-time environment variables to point to the latest testnet's config files.
fn setup_testnet_config() -> anyhow::Result<()> {
    // Get the path to the testnets directory, in a platform-agnostic manner
    let testnets_path = std::env::current_dir()
        .context("could not get current working directory")?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("could not get parent of current working directory"))?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("could not get parent of current working directory"))?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("could not get parent of current working directory"))?
        .join("testnets");

    // Get the most recent testnet name and its configuration directory
    let (latest_testnet_name, latest_testnet_dirname) = latest_testnet(&testnets_path)?;

    let latest_testnet_dirpath = Path::join(&testnets_path, &latest_testnet_dirname);

    // Output the name of the most recent testnet as a build-time environment variable
    println!("cargo:rustc-env=PD_LATEST_TESTNET_NAME={latest_testnet_name}");

    // Ensure that changes to the allocations files trigger a rebuild of pd.
    println!(
        "cargo:rerun-if-changed={}",
        latest_testnet_dirpath.display()
    );

    // For each association of environment variable to filename, set the full path to that file in
    // the environment variable, so that we can include its contents at build time
    for (env_var, filename) in [
        ("PD_LATEST_TESTNET_ALLOCATIONS", "allocations.csv"),
        ("PD_LATEST_TESTNET_VALIDATORS", "validators.json"),
    ] {
        let path = testnets_path.join(&latest_testnet_dirname).join(filename);
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
