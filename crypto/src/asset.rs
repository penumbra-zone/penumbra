//! Asset types and identifiers.

use ark_ff::fields::PrimeField;
use once_cell::sync::Lazy;

use crate::Fq;

/// An identifier for an IBC asset type.
///
/// This is similar to, but different from, the design in [ADR001].  As in
/// ADR001, a denomination trace is hashed to a fixed-size identifier, but
/// unlike ADR001, we hash to a field element rather than a byte string.
///
/// A denomination trace looks like
///
/// - `denom` (native chain A asset)
/// - `transfer/channelToA/denom` (chain B representation of chain A asset)
/// - `transfer/channelToB/transfer/channelToA/denom` (chain C representation of chain B representation of chain A asset)
///
/// ADR001 defines the IBC asset ID as the SHA-256 hash of the denomination
/// trace.  Instead, Penumbra hashes to a field element, so that asset IDs can
/// be more easily used inside of a circuit.
///
/// [ADR001]:
/// https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-001-coin-source-tracing.md
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Id(pub Fq);

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ark_ff::BigInteger;
        let bytes = self.0.into_repr().to_bytes_le();
        f.write_fmt(format_args!("asset::Id({})", hex::encode(&bytes)))
    }
}

// XXX define a DenomTrace structure ?

impl From<&[u8]> for Id {
    fn from(slice: &[u8]) -> Id {
        // Convert an asset name to an asset ID by hashing to a scalar
        Id(Fq::from_le_bytes_mod_order(
            // XXX choice of hash function?
            blake2b_simd::Params::default()
                .personal(b"penumbra.asset")
                .hash(slice)
                .as_bytes(),
        ))
    }
}

/// The domain separator used to hash asset ids to value generators.
static VALUE_GENERATOR_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.value.generator").as_bytes())
});

impl Id {
    /// Compute the value commitment generator for this asset.
    pub fn value_generator(&self) -> decaf377::Element {
        use crate::poseidon_hash::hash_1;
        let hash = hash_1(&VALUE_GENERATOR_DOMAIN_SEP, self.0);
        decaf377::Element::map_to_group_cdh(&hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_up_some_fake_asset_ids() {
        // marked for future deletion
        // not really a test, just a way to exercise the code

        let pen_trace = b"pen";
        let atom_trace = b"HubPort/HubChannel/atom";

        let pen_id = Id::from(&pen_trace[..]);
        let atom_id = Id::from(&atom_trace[..]);

        dbg!(pen_id);
        dbg!(atom_id);

        dbg!(pen_id.value_generator());
        dbg!(atom_id.value_generator());
    }
}
