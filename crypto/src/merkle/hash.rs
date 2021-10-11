use decaf377::Fq;

use crate::merkle::constants::MERKLE_DOMAIN_SEP;
use crate::poseidon_hash::hash_4;

/// Hash into the Merkle tree.
///
/// When we hash elements into the Merkle tree, we always
/// hash the layer this node is at into the domain separator.
pub(crate) fn merkle_hash(layer: u32, inputs: (Fq, Fq, Fq, Fq)) -> Fq {
    let layer_fq: Fq = layer.into();
    let layer_domain_sep: Fq = *MERKLE_DOMAIN_SEP + layer_fq;

    hash_4(&layer_domain_sep, (inputs.0, inputs.1, inputs.2, inputs.3))
}
