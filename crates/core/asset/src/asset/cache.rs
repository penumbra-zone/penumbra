use std::{collections::BTreeMap, ops::Deref, sync::Arc};

use super::{denom_metadata, DenomMetadata, Id, REGISTRY};
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
    pub fn get_by_id(&self, id: Id) -> Option<DenomMetadata> {
        self.cache.get(&id).cloned()
    }

    pub fn get_unit(&self, raw_denom: &str) -> Option<Unit> {
        self.units.get(raw_denom).cloned()
    }

    fn _try_populate(&mut self, _raw_denom: &str) -> anyhow::Result<Option<DenomMetadata>> {
        // First try to parse the raw denom string as a specific denom unit of some kind, to see if already present in the cache

        // Ok(if let Some(unit) = self._get_unit(raw_denom) {
        //     // If the raw denom returns an associated unit, the denom metadata should already be present in the cache as well, so retrieve that.
        //     self._get_by_id(unit.id())
        // } else {
        //     // If the raw denom isn't present in the cache, what should we do here in the typical case? simply return None to indicate population failed?
        //     // In some cases, there will be countless possible denoms of a specific type/in a specific denom family (i.e. all the possible DelegationTokens),
        //     // so we can't reasonably populate the cache with all of them or necessarily provide all metadata for them, but we can & should still parse the raw denom
        //     // to determine whether the newly-witnessed denom is of a type we are familiar with.

        //     // TODO: Before returning None, attempt to parse the raw denom to determine whether it is of a known type.
        //     // If so, return some kind of "empty" or default denom metadata for that type, which can be used to populate the cache with the raw denom as the base denom
        //     // and with whatever useful information we may otherwise be able to interpolate.

        //     // Types:
        //     // - DelegationToken
        //     // - IbcToken
        //     // - UnbondingToken
        //     // - whatever else?

        //     None
        // })

        unimplemented!("try_populate")
    }

    pub fn with_known_assets() -> Self {
        let mut cache = Cache::default();

        let known_assets = vec![
            DenomMetadata {
                inner: Arc::new(denom_metadata::Inner::new(
                    "upenumbra".to_string(),
                    vec![
                        denom_metadata::BareDenomUnit {
                            exponent: 6,
                            denom: "penumbra".to_string(),
                        },
                        denom_metadata::BareDenomUnit {
                            exponent: 3,
                            denom: "mpenumbra".to_string(),
                        },
                    ],
                )),
            },
            DenomMetadata {
                inner: Arc::new(denom_metadata::Inner::new(
                    "ugn".to_string(),
                    vec![
                        denom_metadata::BareDenomUnit {
                            exponent: 6,
                            denom: "gn".to_string(),
                        },
                        denom_metadata::BareDenomUnit {
                            exponent: 3,
                            denom: "mgn".to_string(),
                        },
                    ],
                )),
            },
            DenomMetadata {
                inner: Arc::new(denom_metadata::Inner::new(
                    "ugm".to_string(),
                    vec![
                        denom_metadata::BareDenomUnit {
                            exponent: 6,
                            denom: "gm".to_string(),
                        },
                        denom_metadata::BareDenomUnit {
                            exponent: 3,
                            denom: "mgm".to_string(),
                        },
                    ],
                )),
            },
            DenomMetadata {
                inner: Arc::new(denom_metadata::Inner::new(
                    "wtest_usd".to_string(),
                    vec![denom_metadata::BareDenomUnit {
                        exponent: 6,
                        denom: "test_usd".to_string(),
                    }],
                )),
            },
            DenomMetadata {
                inner: Arc::new(denom_metadata::Inner::new(
                    "test_sat".to_string(),
                    vec![denom_metadata::BareDenomUnit {
                        exponent: 8,
                        denom: "test_btc".to_string(),
                    }],
                )),
            },
            DenomMetadata {
                inner: Arc::new(denom_metadata::Inner::new(
                    "utest_atom".to_string(),
                    vec![
                        denom_metadata::BareDenomUnit {
                            exponent: 6,
                            denom: "test_atom".to_string(),
                        },
                        denom_metadata::BareDenomUnit {
                            exponent: 3,
                            denom: "mtest_atom".to_string(),
                        },
                    ],
                )),
            },
            DenomMetadata {
                inner: Arc::new(denom_metadata::Inner::new(
                    "utest_osmo".to_string(),
                    vec![
                        denom_metadata::BareDenomUnit {
                            exponent: 6,
                            denom: "test_osmo".to_string(),
                        },
                        denom_metadata::BareDenomUnit {
                            exponent: 3,
                            denom: "mtest_osmo".to_string(),
                        },
                    ],
                )),
            },
            DenomMetadata {
                inner: Arc::new(denom_metadata::Inner::new(
                    "uubtc".to_string(),
                    vec![denom_metadata::BareDenomUnit {
                        exponent: 6,
                        denom: "ubtc".to_string(),
                    }],
                )),
            },
            DenomMetadata {
                inner: Arc::new(denom_metadata::Inner::new(
                    "ucube".to_string(),
                    vec![denom_metadata::BareDenomUnit {
                        exponent: 1,
                        denom: "cube".to_string(),
                    }],
                )),
            },
            DenomMetadata {
                inner: Arc::new(denom_metadata::Inner::new(
                    "unala".to_string(),
                    vec![
                        denom_metadata::BareDenomUnit {
                            exponent: 6,
                            denom: "nala".to_string(),
                        },
                        denom_metadata::BareDenomUnit {
                            exponent: 3,
                            denom: "mnala".to_string(),
                        },
                    ],
                )),
            },
        ];

        cache.extend(known_assets);

        cache
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
                self.units.insert(unit.to_string(), unit);
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
