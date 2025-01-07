//! The Penumbra proving and verification key files are binary
//! data that must be provided at build time, so that the key material
//! can be injected into Rust types. The key material is too large, however,
//! for uploading to crates.io (with the keys the crate weights ~100MB).
//!
//! Instead, we'll upload just git raw git-lfs pointer files when publishing to crates.io,
//! then use the build.rs logic to fetch the assets ahead of compilation. Use the feature
//! `download-proving-keys` to enable the auto-download behavior.
use anyhow::Context;
use std::io::Read;

fn main() {
    let proving_parameter_files = [
        "src/gen/output_pk.bin",
        "src/gen/spend_pk.bin",
        "src/gen/swap_pk.bin",
        "src/gen/swapclaim_pk.bin",
        "src/gen/convert_pk.bin",
        "src/gen/delegator_vote_pk.bin",
        "src/gen/nullifier_derivation_pk.bin",
    ];
    let verification_parameter_files = [
        "src/gen/output_vk.param",
        "src/gen/spend_vk.param",
        "src/gen/swap_vk.param",
        "src/gen/swapclaim_vk.param",
        "src/gen/convert_vk.param",
        "src/gen/delegator_vote_vk.param",
        "src/gen/nullifier_derivation_vk.param",
    ];
    for file in proving_parameter_files
        .into_iter()
        .chain(verification_parameter_files)
    {
        println!("cargo:rerun-if-changed={file}");
    }

    for file in proving_parameter_files {
        handle_proving_key(file).expect("failed while handling proving keys");
    }
}

/// Inspect keyfiles, to figure out whether they're git-lfs pointers.
/// If so, and if the `download-proving-keys` feature is set, then fetch
/// the key material over the network via Github API. Otherwise, error
/// out with an informative message.
fn handle_proving_key(file: &str) -> anyhow::Result<()> {
    let r = ProvingKeyFilepath::new(file);
    match r {
        ProvingKeyFilepath::Present(_f) => {}
        ProvingKeyFilepath::Absent(f) => {
            println!(
                "cargo:warning=proving key file is missing: {} this should not happen",
                f
            );
            anyhow::bail!(
                "proving key file not found; at least lfs pointers were expected; path={}",
                f
            );
        }
        ProvingKeyFilepath::Pointer(f) => {
            #[cfg(feature = "download-proving-keys")]
            download_proving_key(&f)?;
            #[cfg(not(feature = "download-proving-keys"))]
            println!(
                "cargo:warning=proving key file is lfs pointer: {} enable 'download-proving-keys' feature to obtain key files",
                f
            );
        }
    }
    Ok(())
}

/// The states that a proving key filepath can be in.
enum ProvingKeyFilepath {
    /// The filepath does not exist.
    ///
    /// `Absent` is the expected state when building from crates.io,
    /// because the binary keyfiles are excluded from the crate manifest, due to filesize.
    /// If the keyfiles were bundled into the crate, it'd be ~100MB, far too large for crates.io.
    Absent(String),

    /// The filepath was found, but appears to be a git-lfs pointer.
    ///
    /// `Pointer` is the expected state when:
    ///
    ///   * building from source, via a local git checkout, but without git-lfs being configured;
    ///   * building from crates.io, because only the git-lfs pointers were uploaded
    ///
    /// If the `download-proving-keys` feature is set, then the proving keys will be fetched
    /// via the Github LFS API and written in place in the source checkout. Otherwise,
    /// an error is thrown.
    Pointer(String),

    /// The filepath was found, and appears to be a fully-fleged binary key file.
    ///
    /// `Present` is the expected state when building from source, via a local git checkout,
    /// with git-lfs properly configured.
    Present(String),
}

impl ProvingKeyFilepath {
    fn new(filepath: &str) -> Self {
        if std::fs::metadata(filepath).is_ok() {
            let bytes = file_to_bytes(filepath).expect("failed to read filepath as bytes");
            // If the file is smaller than 500 bytes, we'll assume it's an LFS pointer.
            if bytes.len() < 500 {
                ProvingKeyFilepath::Pointer(filepath.into())
            } else {
                ProvingKeyFilepath::Present(filepath.into())
            }
        } else {
            ProvingKeyFilepath::Absent(filepath.into())
        }
    }
}

/// Read filepath to byte array.
fn file_to_bytes(filepath: &str) -> anyhow::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let f = std::fs::File::open(filepath)
        .with_context(|| "can open proving key file from local source")?;
    let mut reader = std::io::BufReader::new(f);
    reader
        .read_to_end(&mut bytes)
        .with_context(|| "can read proving key file")?;
    Ok(bytes)
}

#[cfg(feature = "download-proving-keys")]
pub fn download_proving_key(filepath: &str) -> anyhow::Result<()> {
    use std::io::Write;

    let bytes = file_to_bytes(filepath)?;
    let pointer =
        downloads::GitLFSPointer::parse(&bytes[..]).with_context(|| "can parse pointer")?;
    let downloaded_bytes = pointer
        .resolve()
        .with_context(|| "can download proving key from git-lfs")?;

    // Save downloaded bytes to file.
    let f =
        std::fs::File::create(filepath).with_context(|| "can open downloaded proving key file")?;
    let mut writer = std::io::BufWriter::new(f);
    writer
        .write_all(&downloaded_bytes[..])
        .with_context(|| "can write downloaded proving key to local file")?;
    Ok(())
}

#[cfg(feature = "download-proving-keys")]
mod downloads {
    use anyhow::Context;
    use regex::Regex;
    use reqwest::blocking::Client;

    /// The Git LFS server to use.
    static GIT_LFS_SERVER: &str =
        "https://github.com/penumbra-zone/penumbra.git/info/lfs/objects/batch";

    /// Represents a Git LFS pointer.
    pub struct GitLFSPointer {
        /// The unique object ID.
        oid: String,
        /// The hash algorithm used to compute the OID. Only `sha256` is supported.
        hash_algo: String,
        /// The size of the object in bytes.
        size: usize,
    }

    impl GitLFSPointer {
        /// Parses a Git LFS pointer from raw bytes.
        pub fn parse(bytes: &[u8]) -> anyhow::Result<Self> {
            let pointer_utf8 =
                std::str::from_utf8(bytes).with_context(|| "git LFS should be valid UTF-8")?;

            // `oid sha256:digest`
            let oid_re = Regex::new(r"oid [\w,:]*").unwrap();
            let caps = oid_re
                .captures(pointer_utf8)
                .with_context(|| "git LFS pointers should have oid field")?;
            let oid_line: Vec<String> = caps
                .get(0)
                .with_context(|| "hash algorithm should be in oid field")?
                .as_str()
                .split_whitespace()
                .map(str::to_owned)
                .collect();
            let hash_and_oid: Vec<String> = oid_line[1].split(':').map(str::to_owned).collect();
            let hash_algo = hash_and_oid[0].clone();
            let oid = hash_and_oid[1].clone();

            // `size 12345`
            let size_re = Regex::new(r"size [0-9]*").unwrap();
            let caps = size_re
                .captures(pointer_utf8)
                .with_context(|| "git LFS pointers have size field")?;
            let size_line: Vec<String> = caps
                .get(0)
                .with_context(|| "size in bytes should be in git LFS pointer")?
                .as_str()
                .split_whitespace()
                .map(str::to_owned)
                .collect();
            let size = size_line[1]
                .parse()
                .with_context(|| "size should be a number")?;

            Ok(Self {
                oid,
                hash_algo,
                size,
            })
        }

        /// Resolves the pointer using the Git LFS Batch API.
        /// https://github.com/git-lfs/git-lfs/blob/main/docs/api/batch.md
        pub fn resolve(&self) -> anyhow::Result<Vec<u8>> {
            // Download using Git LFS Batch API
            let request_body = format!(
                r#"{{"operation": "download", "transfer": ["basic"], "objects": [{{"oid": "{}", "size": {}}}]}}"#,
                self.oid, self.size
            );
            let client = Client::new();
            let res = client
                .post(GIT_LFS_SERVER)
                .header("Accept", "application/vnd.git-lfs+json")
                .header("Content-type", "application/vnd.git-lfs+json")
                .body(request_body)
                .send()
                .with_context(|| "can get response from Git LFS server")?;

            // JSON response contains "objects" array -> 0 -> "actions" -> "download" -> "href" which has the
            // actual location of the file.
            let json_res = res
                .json::<serde_json::Value>()
                .with_context(|| "result is JSON formatted")?;

            let href = json_res
                .get("objects")
                .with_context(|| "objects key exists")?
                .get(0)
                .with_context(|| "has at least one entry")?
                .get("actions")
                .with_context(|| "has actions key")?
                .get("download")
                .with_context(|| "has download key")?
                .get("href")
                .with_context(|| "has href key")?
                .as_str()
                .with_context(|| "can get href from Git LFS response")?;

            // Actually download that file using the provided URL.
            let res = client.get(href).send().with_context(|| "can get file")?;
            let bytes = res.bytes().with_context(|| "can get bytes from file")?;

            // Check hash locally.
            if self.hash_algo != "sha256" {
                unimplemented!("only sha256 is supported");
            } else {
                use sha2::{Digest, Sha256};
                let sha256_digest = Sha256::digest(&bytes);
                let sha256_str = hex::encode(sha256_digest);
                assert_eq!(sha256_str, self.oid);
            }

            Ok(bytes.into())
        }
    }
}
