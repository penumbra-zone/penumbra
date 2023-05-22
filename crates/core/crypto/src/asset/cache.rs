use std::{collections::BTreeMap, ops::Deref};

use super::{DenomMetadata, Id, REGISTRY};
use crate::asset::denom_metadata::Unit;

/// On-chain data structures only record a fixed-size [`Id`], so this type
/// allows caching known [`BaseDenom`]s.
///
/// The cache is backed by a [`BTreeMap`] accessed through a [`Deref`] impl.
///
/// For (de)serialization, [`From`] conversions are provided to a `BTreeMap<Id,
/// String>` with the string representations of the base denominations.
#[derive(Clone, Default, Debug)]
pub struct Cache {
    cache: BTreeMap<Id, DenomMetadata>,
    units: BTreeMap<String, Unit>,
}

impl Cache {
    fn _get_by_id(&self, id: Id) -> Option<DenomMetadata> {
        self.cache.get(&id).cloned()
    }

    fn _get_unit(&self, raw_denom: &str) -> Option<Unit> {
        self.units.get(raw_denom).cloned()
    }
}

// Implementing Deref but not DerefMut means people get unlimited read access,
// but can only write into the cache through approved methods.
impl Deref for Cache {
    type Target = BTreeMap<Id, DenomMetadata>;

    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}

impl From<Cache> for BTreeMap<Id, DenomMetadata> {
    fn from(cache: Cache) -> Self {
        cache
            .cache
            .into_iter()
            .map(|(id, denom)| (id, denom))
            .collect()
    }
}

impl TryFrom<BTreeMap<Id, DenomMetadata>> for Cache {
    type Error = anyhow::Error;

    fn try_from(map: BTreeMap<Id, DenomMetadata>) -> Result<Self, Self::Error> {
        let mut cache = BTreeMap::default();
        let mut units: BTreeMap<String, Unit> = BTreeMap::default();
        for (provided_id, denom) in map.into_iter() {
            if let Some(denom) = REGISTRY.parse_denom(&denom.base_denom().denom) {
                let id = denom.id();
                if provided_id != id {
                    return Err(anyhow::anyhow!(
                        "provided id {} for denom {} does not match computed id {}",
                        provided_id,
                        denom,
                        id
                    ));
                }
                cache.insert(id, denom.clone());
                units.insert(denom.base_denom().denom, denom.base_unit());
            } else {
                return Err(anyhow::anyhow!(
                    "invalid base denom {}",
                    denom.base_denom().denom
                ));
            }
        }
        Ok(Self { cache, units })
    }
}

// BaseDenom already has a validated Id, so by implementing Extend<BaseDenom> we
// can ensure we don't insert any invalid Ids
impl Extend<DenomMetadata> for Cache {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = DenomMetadata>,
    {
        for denom in iter {
            let id = denom.id();
            self.cache.insert(id, denom.clone());

            for unit in denom.units() {
                self.units.insert(unit.base().base_denom().denom, unit);
            }
        }
    }
}

impl FromIterator<DenomMetadata> for Cache {
    fn from_iter<T: IntoIterator<Item = DenomMetadata>>(iter: T) -> Self {
        let mut cache = Cache::default();
        cache.extend(iter);
        cache
    }
}
