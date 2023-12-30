use penumbra_tct as tct;
use penumbra_txhash::EffectHash;

/// Stateless verification context for a transaction.
///
/// TODO: this is located in this crate just for convenience (at the bottom of the dep tree).
#[derive(Clone, Debug)]
pub struct TransactionContext {
    /// The transaction's anchor.
    pub anchor: tct::Root,
    /// The transaction's effect hash.
    pub effect_hash: EffectHash,
}
