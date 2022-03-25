use hash_hasher::{HashedMap, HashedSet};
use penumbra_tct::{
    block::InsertError, internal::hash::Hash, Commitment, Position, Proof, Witness,
};

use super::{Tier, Tree};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Builder {
    pub tiers: Tier<Tier<Commitment>>,
}
