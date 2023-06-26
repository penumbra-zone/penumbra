use anyhow::Context;
use std::path::PathBuf;
use std::process::Command;

const PROTOC_TARGET_MAJOR_VERSION: i32 = 23;
const DEV_DOCS_URL: &str = "https://guide.penumbra.zone/main/dev/protobuf.html";

/// Inspect local version of `protoc` binary and ensure it's compatible
/// with the supported major version. We do this to ensure that the
/// binary outputs are generated stably across many workstation setups.
fn check_protoc_version() -> anyhow::Result<()> {
    let output = Command::new("protoc")
        .args(["--version"])
        .output()
        .context(format!(
            "Could not find protoc. Is it installed? See dev docs at {}",
            DEV_DOCS_URL
        ))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout_parts = stdout.trim().split(' ').collect::<Vec<&str>>();
    let version = stdout_parts.last().unwrap();
    let version_parts = version.split('.').collect::<Vec<&str>>();
    let major_version: i32 = version_parts.first().unwrap().parse().unwrap();
    if major_version != PROTOC_TARGET_MAJOR_VERSION {
        let msg = format!("This tool expects protoc version {PROTOC_TARGET_MAJOR_VERSION}.x, but {version} is installed locally.\nPlease install a compatible version. For more info, see {DEV_DOCS_URL}");
        anyhow::bail!(msg);
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    check_protoc_version()?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("{}", root.display());
    let target_dir = root
        .join("..")
        .join("..")
        .join("crates")
        .join("proto")
        .join("src")
        .join("gen");
    println!("{}", target_dir.display());

    let descriptor_path = target_dir.join("proto_descriptor.bin");

    let mut config = prost_build::Config::new();
    config.out_dir(&target_dir);
    config
        .file_descriptor_set_path(&descriptor_path)
        .compile_well_known_types();

    // Specify which parts of the protos should have their `bytes` fields
    // converted to Rust `Bytes` (= zero-copy view into a shared buffer) rather
    // than `Vec<u8>`.
    //
    // The upside of using the `Bytes` type is that it avoids copies while
    // parsing the protos.
    //
    // The downside is that the underlying buffer is kept alive as long as there
    // is at least one view into it, so holding on to a small amount of data
    // (e.g., a hash) could keep a much larger buffer live for a long time,
    // increasing memory use.
    //
    // Getting this tradeoff perfect isn't essential, but it's useful to keep in mind.
    config.bytes(&[
        // Transactions have a lot of `bytes` fields that need to be converted
        // into fixed-size byte arrays anyways, so there's no point allocating
        // into a temporary vector.
        ".penumbra.core.transaction",
        // The byte fields in a compact block will also be converted to fixed-size
        // byte arrays and then discarded.
        ".penumbra.core.crypto.v1alpha1.NotePayload",
        ".penumbra.core.chain.v1alpha1.CompactBlock",
    ]);

    // As recommended in pbjson_types docs.
    config.extern_path(".google.protobuf", "::pbjson_types");
    // NOTE: we need this because the rust module that defines the IBC types is external, and not
    // part of this crate.
    // See https://docs.rs/prost-build/0.5.0/prost_build/struct.Config.html#method.extern_path
    config.extern_path(".ibc", "::ibc_proto::ibc");
    config.extern_path(".ics23", "::ics23");
    // The same applies for some of the tendermint types.
    // config.extern_path(
    //     ".tendermint.types.Validator",
    //     "::tendermint::types::Validator",
    // );
    // config.extern_path(
    //     ".tendermint.p2p.DefaultNodeInfo",
    //     "::tendermint::p2p::DefaultNodeInfo",
    // );
    //

    config.compile_protos(
        &[
            "../../proto/penumbra/penumbra/core/crypto/v1alpha1/crypto.proto",
            "../../proto/penumbra/penumbra/core/transaction/v1alpha1/transaction.proto",
            "../../proto/penumbra/penumbra/core/stake/v1alpha1/stake.proto",
            "../../proto/penumbra/penumbra/core/chain/v1alpha1/chain.proto",
            "../../proto/penumbra/penumbra/core/ibc/v1alpha1/ibc.proto",
            "../../proto/penumbra/penumbra/core/dex/v1alpha1/dex.proto",
            "../../proto/penumbra/penumbra/core/transparent_proofs/v1alpha1/transparent_proofs.proto",
            "../../proto/penumbra/penumbra/core/governance/v1alpha1/governance.proto",
            "../../proto/rust-vendored/tendermint/types/validator.proto",
            "../../proto/rust-vendored/tendermint/p2p/types.proto",
        ],
        &["../../proto/penumbra/", "../../proto/rust-vendored/"],
    )?;

    // For the client code, we also want to generate RPC instances, so compile via tonic:
    tonic_build::configure()
        .out_dir(&target_dir)
        .server_mod_attribute("penumbra.client.v1alpha1", "#[cfg(feature = \"rpc\")]")
        .client_mod_attribute("penumbra.client.v1alpha1", "#[cfg(feature = \"rpc\")]")
        .server_mod_attribute("penumbra.view.v1alpha1", "#[cfg(feature = \"rpc\")]")
        .client_mod_attribute("penumbra.view.v1alpha1", "#[cfg(feature = \"rpc\")]")
        .server_mod_attribute("penumbra.custody.v1alpha1", "#[cfg(feature = \"rpc\")]")
        .client_mod_attribute("penumbra.custody.v1alpha1", "#[cfg(feature = \"rpc\")]")
        .server_mod_attribute(
            "penumbra.narsil.ledger.v1alpha1",
            "#[cfg(feature = \"rpc\")]",
        )
        .client_mod_attribute(
            "penumbra.narsil.ledger.v1alpha1",
            "#[cfg(feature = \"rpc\")]",
        )
        .server_mod_attribute(
            "cosmos.base.tendermint.v1beta1",
            "#[cfg(feature = \"rpc\")]",
        )
        .client_mod_attribute(
            "cosmos.base.tendermint.v1beta1",
            "#[cfg(feature = \"rpc\")]",
        )
        .compile_with_config(
            config,
            &[
                "../../proto/penumbra/penumbra/client/v1alpha1/client.proto",
                "../../proto/penumbra/penumbra/narsil/ledger/v1alpha1/ledger.proto",
                "../../proto/penumbra/penumbra/view/v1alpha1/view.proto",
                "../../proto/penumbra/penumbra/custody/v1alpha1/custody.proto",
                "../../proto/rust-vendored/tendermint/types/validator.proto",
                "../../proto/rust-vendored/tendermint/p2p/types.proto",
            ],
            &["../../proto/penumbra/", "../../proto/rust-vendored/"],
        )?;

    // Finally, build pbjson Serialize, Deserialize impls:
    let descriptor_set = std::fs::read(descriptor_path)?;
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)?
        .out_dir(&target_dir)
        // These are all excluded because they're part of the Tendermint proxy,
        // so they use `tendermint` types that may not be Serialize/Deserialize,
        // and we don't need to serialize them with Serde anyways.
        .exclude([
            ".penumbra.client.v1alpha1.ABCIQueryResponse".to_owned(),
            ".penumbra.client.v1alpha1.GetBlockByHeightResponse".to_owned(),
            ".penumbra.client.v1alpha1.GetStatusResponse".to_owned(),
        ])
        .build(&[".penumbra"])?;

    Ok(())
}
