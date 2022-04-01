use std::io::Result;

fn main() -> Result<()> {
    let mut config = prost_build::Config::new();

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
        ".penumbra.transaction",
        // The byte fields in a compact block will also be converted to fixed-size
        // byte arrays and then discarded.
        ".penumbra.chain.CompactOutput",
        ".penumbra.chain.CompactBlock",
    ]);

    for (path, attribute) in TYPE_ATTRIBUTES.iter() {
        config.type_attribute(path, attribute);
    }
    for (path, attribute) in FIELD_ATTRIBUTES.iter() {
        config.field_attribute(path, attribute);
    }

    // NOTE: we need this because the rust module that defines the IBC types is external, and not
    // part of this crate.
    // See https://docs.rs/prost-build/0.5.0/prost_build/struct.Config.html#method.extern_path
    config.extern_path(".ibc", "::ibc_proto::ibc");

    config.compile_protos(
        &[
            "proto/crypto.proto",
            "proto/transaction.proto",
            "proto/stake.proto",
            "proto/chain.proto",
            "proto/genesis.proto",
            "proto/ibc.proto",
        ],
        &["proto/", "ibc-go-vendor/"],
    )?;

    // These should disappear, eventually.
    config.compile_protos(
        &["proto/transparent_proofs.proto", "proto/sighash.proto"],
        &["proto/", "ibc-go-vendor/"],
    )?;

    // For the client code, we also want to generate RPC instances, so compile via tonic:
    tonic_build::configure().compile_with_config(
        config,
        &["proto/light_wallet.proto", "proto/thin_wallet.proto"],
        &["proto/", "ibc-go-vendor/"],
    )?;

    Ok(())
}

static SERIALIZE: &str = r#"#[derive(::serde::Deserialize, ::serde::Serialize)]"#;
/// Serializes newtype structs as if the inner field were serialized on its own.
static SERDE_TRANSPARENT: &str = r#"#[serde(transparent)]"#;

static AS_HEX: &str = r#"#[serde(with = "crate::serializers::hexstr")]"#;
static AS_BASE64: &str = r#"#[serde(with = "crate::serializers::base64str")]"#;
static AS_BECH32_IDENTITY_KEY: &str =
    r#"#[serde(with = "crate::serializers::bech32str::validator_identity_key")]"#;
static AS_BECH32_ADDRESS: &str = r#"#[serde(with = "crate::serializers::bech32str::address")]"#;
static AS_BECH32_ASSET_ID: &str = r#"#[serde(with = "crate::serializers::bech32str::asset_id")]"#;

static TYPE_ATTRIBUTES: &[(&str, &str)] = &[
    (".penumbra.stake.Validator", SERIALIZE),
    (".penumbra.stake.FundingStream", SERIALIZE),
    (".penumbra.stake.ValidatorDefinition", SERIALIZE),
    (".penumbra.stake.ValidatorInfo", SERIALIZE),
    (".penumbra.stake.ValidatorJMTKeys", SERIALIZE),
    (".penumbra.stake.ValidatorState", SERIALIZE),
    (".penumbra.stake.ValidatorStateName", SERIALIZE),
    (".penumbra.stake.ValidatorStatus", SERIALIZE),
    (".penumbra.stake.RateData", SERIALIZE),
    (".penumbra.stake.BaseRateData", SERIALIZE),
    (".penumbra.stake.IdentityKey", SERIALIZE),
    (".penumbra.stake.IdentityKey", SERDE_TRANSPARENT),
    (".penumbra.stake.Delegate", SERIALIZE),
    (".penumbra.stake.Undelegate", SERIALIZE),
    (".penumbra.crypto.Address", SERIALIZE),
    (".penumbra.crypto.Address", SERDE_TRANSPARENT),
    (".penumbra.crypto.NoteCommitment", SERIALIZE),
    (".penumbra.crypto.NoteCommitment", SERDE_TRANSPARENT),
    (".penumbra.crypto.AssetId", SERIALIZE),
    (".penumbra.crypto.AssetId", SERDE_TRANSPARENT),
    (".penumbra.crypto.Value", SERIALIZE),
    (".penumbra.crypto.Denom", SERIALIZE),
    (".penumbra.crypto.Denom", SERDE_TRANSPARENT),
    (".penumbra.crypto.MerkleRoot", SERIALIZE),
    (".penumbra.crypto.MerkleRoot", SERDE_TRANSPARENT),
    (".penumbra.chain.ChainParams", SERIALIZE),
    (".penumbra.chain.CompactBlock", SERIALIZE),
    (".penumbra.chain.CompactOutput", SERIALIZE),
    (".penumbra.genesis.GenesisAppState", SERIALIZE),
    (".penumbra.genesis.Allocation", SERIALIZE),
    (".penumbra.genesis.ValidatorPower", SERIALIZE),
    (".penumbra.transaction.OutputBody", SERIALIZE),
];

static FIELD_ATTRIBUTES: &[(&str, &str)] = &[
    // Using base64 for the validator's consensus key means that
    // the format is the same as the Tendermint json config files.
    (".penumbra.stake.Validator.consensus_key", AS_BASE64),
    (".penumbra.stake.ValidatorDefinition.auth_sig", AS_HEX),
    (".penumbra.stake.IdentityKey.ik", AS_BECH32_IDENTITY_KEY),
    (".penumbra.crypto.Address.inner", AS_BECH32_ADDRESS),
    (".penumbra.crypto.AssetId.inner", AS_BECH32_ASSET_ID),
    (".penumbra.crypto.NoteCommitment.inner", AS_HEX),
    (".penumbra.crypto.MerkleRoot.inner", AS_HEX),
];
