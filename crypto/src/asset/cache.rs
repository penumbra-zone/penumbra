use std::{collections::BTreeMap, ops::Deref};

use super::{BaseDenom, Id, REGISTRY};

/// On-chain data structures only record a fixed-size [`Id`], so this type
/// allows caching known [`BaseDenom`]s.
///
/// The cache is backed by a [`BTreeMap`] accessed through a [`Deref`] impl.
///
/// For (de)serialization, [`From`] conversions are provided to a `BTreeMap<Id,
/// String>` with the string representations of the base denominations.
#[derive(Clone, Default, Debug)]
pub struct Cache(BTreeMap<Id, BaseDenom>);

// Implementing Deref but not DerefMut means people get unlimited read access,
// but can only write into the cache through approved methods.
impl Deref for Cache {
    type Target = BTreeMap<Id, BaseDenom>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Cache> for BTreeMap<Id, String> {
    fn from(cache: Cache) -> Self {
        cache
            .0
            .into_iter()
            .map(|(id, denom)| (id, denom.to_string()))
            .collect()
    }
}

impl TryFrom<BTreeMap<Id, String>> for Cache {
    type Error = anyhow::Error;

    fn try_from(map: BTreeMap<Id, String>) -> Result<Self, Self::Error> {
        let mut cache = BTreeMap::default();
        for (provided_id, denom_str) in map.into_iter() {
            if let Some(denom) = REGISTRY.parse_base(&denom_str) {
                let id = denom.id();
                if provided_id != id {
                    return Err(anyhow::anyhow!(
                        "provided id {} for denom {} does not match computed id {}",
                        provided_id,
                        denom,
                        id
                    ));
                }
                cache.insert(id, denom);
            } else {
                return Err(anyhow::anyhow!("invalid base denom {}", denom_str));
            }
        }
        Ok(Self(cache))
    }
}

// BaseDenom already has a validated Id, so by implementing Extend<BaseDenom> we
// can ensure we don't insert any invalid Ids
impl Extend<BaseDenom> for Cache {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = BaseDenom>,
    {
        self.0
            .extend(iter.into_iter().map(|denom| (denom.id(), denom)));
    }
}

impl FromIterator<BaseDenom> for Cache {
    fn from_iter<T: IntoIterator<Item = BaseDenom>>(iter: T) -> Self {
        let mut cache = Cache::default();
        cache.extend(iter);
        cache
    }
}
