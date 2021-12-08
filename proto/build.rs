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

    config.compile_protos(&["proto/transaction.proto"], &["proto/"])?;
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
