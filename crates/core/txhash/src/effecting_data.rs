use crate::EffectHash;

/// Something that can be hashed to produce an [`EffectHash`].
pub trait EffectingData {
    fn effect_hash(&self) -> EffectHash;
}
