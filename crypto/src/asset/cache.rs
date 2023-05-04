use std::{collections::BTreeMap, ops::Deref};

use super::{denom::DenomMetadata, Id, REGISTRY};
use crate::asset::Unit;

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
    units: BTreeMap<Id, Unit>,
}

impl Cache {
    fn get_by_id(&self, id: Id) -> Option<DenomMetadata> {
        self.cache.get(&id).cloned()
    }

    fn get_unit(&self, id: Id) -> Option<Unit> {
        self.units.get(&id).cloned()
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

impl From<Cache> for BTreeMap<Id, String> {
    fn from(cache: Cache) -> Self {
        cache
            .cache
            .into_iter()
            .map(|(id, denom)| (id, denom.name))
            .collect()
    }
}

impl TryFrom<BTreeMap<Id, String>> for Cache {
    type Error = anyhow::Error;

    fn try_from(map: BTreeMap<Id, String>) -> Result<Self, Self::Error> {
        let mut cache = BTreeMap::default();
        let mut units: BTreeMap<Id, Unit> = BTreeMap::default();

        for (provided_id, denom_str) in map.into_iter() {
            if let Some(denom) = REGISTRY.parse_denom(&denom_str) {
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
                units.insert(id, denom.base_unit());
            } else {
                return Err(anyhow::anyhow!("invalid base denom {}", denom_str));
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
            let id = denom.penumbra_asset_id;
            self.cache.insert(id, denom.clone());

            for denom_unit in denom.denom_units {
                self.units.insert(id, denom_unit);
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
