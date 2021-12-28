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
        ".penumbra.light_wallet.StateFragment",
        ".penumbra.light_wallet.CompactBlock",
    ]);

    for (path, attribute) in TYPE_ATTRIBUTES.iter() {
        config.type_attribute(path, attribute);
    }
    for (path, attribute) in FIELD_ATTRIBUTES.iter() {
        config.field_attribute(path, attribute);
    }

    config.compile_protos(&["proto/transaction.proto"], &["proto/"])?;
    config.compile_protos(&["proto/stake.proto"], &["proto/"])?;

    // These should disappear, eventually.
    config.compile_protos(&["proto/transparent_proofs.proto"], &["proto/"])?;
    config.compile_protos(&["proto/sighash.proto"], &["proto/"])?;

    // For the client code, we also want to generate RPC instances, so compile via tonic:
    tonic_build::configure().compile_with_config(
        config,
        &["proto/light_wallet.proto", "proto/thin_wallet.proto"],
        &["proto/"],
    )?;

    Ok(())
}

// WARNING: any type attributes adding SERDE_AS **MUST** be placed **BEFORE**
// SERIALIZE. Otherwise, serde_as is applied to the output of the derive macro
// (unstable), rather than the other way around.
// This is a moot point for now, since we can't use field attributes for macro reasons
// static SERDE_AS: &str = r#"#[::serde_with::serde_as]"#;
static SERIALIZE: &str = r#"#[derive(::serde::Deserialize, ::serde::Serialize)]"#;

// Requires SERDE_AS on the container
// :(
// error: expected non-macro attribute, found attribute macro `::serde_with::serde_as`
// use tendermint-rs approach instead?
// https://github.com/penumbra-zone/tendermint-rs/blob/master/proto/src/serializers/bytes.rs#L4-L30
// static AS_HEX: &str = r#"#[::serde_with::serde_as(as = "::serde_with::hex::Hex")]"#;

static AS_HEX: &str = r#"#[serde(with = "crate::serializers::hexstr")]"#;
static AS_BASE64: &str = r#"#[serde(with = "crate::serializers::base64str")]"#;

static TYPE_ATTRIBUTES: &[(&str, &str)] = &[
    //(".penumbra.stake.Validator", SERDE_AS),
    (".penumbra.stake.Validator", SERIALIZE),
    (".penumbra.stake.FundingStream", SERIALIZE),
    //(".penumbra.stake.ValidatorDefinition", SERDE_AS),
    (".penumbra.stake.ValidatorDefinition", SERIALIZE),
    (".penumbra.stake.RateData", SERIALIZE),
    (".penumbra.stake.BaseRateData", SERIALIZE),
];

static FIELD_ATTRIBUTES: &[(&str, &str)] = &[
    // TODO: use bech32 here?
    (".penumbra.stake.Validator.identity_key", AS_HEX),
    // Using base64 for the validator's consensus key means that
    // the format is the same as the Tendermint json config files.
    (".penumbra.stake.Validator.consensus_key", AS_BASE64),
    (".penumbra.stake.ValidatorDefinition.auth_sig", AS_HEX),
    (".penumbra.stake.RateData.identity_key", AS_HEX),
];
