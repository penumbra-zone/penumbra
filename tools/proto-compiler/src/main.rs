use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("{}", root.display());
    let target_dir = root
        .join("..")
        .join("..")
        .join("proto")
        .join("src")
        .join("gen");
    println!("{}", target_dir.display());

    let tmpdir = tempfile::tempdir().unwrap();
    let descriptor_path = tmpdir.path().to_owned().join("proto_descriptor.bin");

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
            "../../proto/proto/penumbra/core/crypto/v1alpha1/crypto.proto",
            "../../proto/proto/penumbra/core/transaction/v1alpha1/transaction.proto",
            "../../proto/proto/penumbra/core/stake/v1alpha1/stake.proto",
            "../../proto/proto/penumbra/core/chain/v1alpha1/chain.proto",
            "../../proto/proto/penumbra/core/ibc/v1alpha1/ibc.proto",
            "../../proto/proto/penumbra/core/dex/v1alpha1/dex.proto",
            "../../proto/proto/penumbra/core/transparent_proofs/v1alpha1/transparent_proofs.proto",
            "../../proto/proto/penumbra/core/governance/v1alpha1/governance.proto",
            "../../proto/rust-vendored/tendermint/types/validator.proto",
            "../../proto/rust-vendored/tendermint/p2p/types.proto",
        ],
        &["../../proto/proto/", "../../proto/rust-vendored/"],
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
                "../../proto/proto/penumbra/client/v1alpha1/client.proto",
                "../../proto/proto/penumbra/narsil/ledger/v1alpha1/ledger.proto",
                "../../proto/proto/penumbra/view/v1alpha1/view.proto",
                "../../proto/proto/penumbra/custody/v1alpha1/custody.proto",
                "../../proto/rust-vendored/cosmos/base/tendermint/v1beta1/query.proto",
                "../../proto/rust-vendored/tendermint/types/validator.proto",
                "../../proto/rust-vendored/tendermint/p2p/types.proto",
            ],
            &["../../proto/proto/", "../../proto/rust-vendored/"],
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
