use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use penumbra_crypto::{IdentityKey, Nullifier};
use penumbra_proto::{core::chain::v1alpha1 as pb, Protobuf};

use crate::sync::AnnotatedNotePayload;

/// All the things scheduled for unquarantine, grouped by unbonding epoch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(into = "pb::Quarantined", try_from = "pb::Quarantined")]
pub struct Quarantined {
    pub quarantined: BTreeMap<u64, Scheduled>,
}

impl Quarantined {
    /// Schedule a note for unquarantine at a given epoch, tied to a given validator.
    pub fn schedule_note(
        &mut self,
        epoch: u64,
        identity_key: IdentityKey,
        note_payload: AnnotatedNotePayload,
    ) {
        self.quarantined
            .entry(epoch)
            .or_default()
            .schedule_note(identity_key, note_payload)
    }

    /// Schedule a nullifier for unquarantine at a given epoch, tied to a given validator.
    pub fn schedule_nullifier(
        &mut self,
        epoch: u64,
        identity_key: IdentityKey,
        nullifier: Nullifier,
    ) {
        self.quarantined
            .entry(epoch)
            .or_default()
            .schedule_nullifier(identity_key, nullifier)
    }

    /// Check if anything is currently quarantined.
    pub fn is_empty(&self) -> bool {
        self.quarantined.is_empty()
    }

    /// Get an iterator over all quarantined epochs.
    pub fn iter(&self) -> impl Iterator<Item = (&u64, &Scheduled)> {
        self.quarantined.iter()
    }
}

impl IntoIterator for Quarantined {
    type Item = <BTreeMap<u64, Scheduled> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<u64, Scheduled> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.quarantined.into_iter()
    }
}

impl<'a> IntoIterator for &'a Quarantined {
    type Item = <&'a BTreeMap<u64, Scheduled> as IntoIterator>::Item;
    type IntoIter = <&'a BTreeMap<u64, Scheduled> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.quarantined.iter()
    }
}

impl Extend<(u64, Scheduled)> for Quarantined {
    fn extend<T: IntoIterator<Item = (u64, Scheduled)>>(&mut self, iter: T) {
        for (epoch, scheduled) in iter {
            self.quarantined.entry(epoch).or_default().extend(scheduled);
        }
    }
}

impl FromIterator<(u64, Scheduled)> for Quarantined {
    fn from_iter<T: IntoIterator<Item = (u64, Scheduled)>>(iter: T) -> Self {
        let mut quarantined = Quarantined::default();
        quarantined.extend(iter);
        quarantined
    }
}

impl Protobuf<pb::Quarantined> for Quarantined {}

impl TryFrom<pb::Quarantined> for Quarantined {
    type Error = anyhow::Error;

    fn try_from(value: pb::Quarantined) -> Result<Self, Self::Error> {
        Ok(Self {
            quarantined: value
                .per_epoch
                .into_iter()
                .map(
                    |pb::quarantined::EpochEntry {
                         unbonding_epoch,
                         scheduled,
                     }| {
                        Ok::<_, anyhow::Error>((
                            unbonding_epoch,
                            scheduled
                                .map(TryInto::try_into)
                                .transpose()?
                                .unwrap_or_default(),
                        ))
                    },
                )
                .collect::<Result<BTreeMap<_, _>, _>>()?,
        })
    }
}

impl From<Quarantined> for pb::Quarantined {
    fn from(value: Quarantined) -> Self {
        Self {
            per_epoch: value
                .quarantined
                .into_iter()
                .map(|(epoch, scheduled)| pb::quarantined::EpochEntry {
                    unbonding_epoch: epoch,
                    scheduled: if scheduled.is_empty() {
                        None
                    } else {
                        Some(scheduled.into())
                    },
                })
                .collect(),
        }
    }
}

/// All the things scheduled for unquarantine in a single (not specified here) epoch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(
    into = "pb::quarantined::Scheduled",
    try_from = "pb::quarantined::Scheduled"
)]
pub struct Scheduled {
    pub scheduled: BTreeMap<IdentityKey, Unbonding>,
}

impl Scheduled {
    /// Schedule a note for unquarantine, tied to a given validator.
    pub fn schedule_note(&mut self, identity_key: IdentityKey, note_payload: AnnotatedNotePayload) {
        self.scheduled
            .entry(identity_key)
            .or_default()
            .schedule_note(note_payload)
    }

    /// Schedule a nullifier for unquarantine, tied to a given validator.
    pub fn schedule_nullifier(&mut self, identity_key: IdentityKey, nullifier: Nullifier) {
        self.scheduled
            .entry(identity_key)
            .or_default()
            .schedule_nullifier(nullifier)
    }

    /// Remove all things quarantined relative to the given validator.
    pub fn unschedule_validator(&mut self, identity_key: IdentityKey) -> Unbonding {
        self.scheduled.remove(&identity_key).unwrap_or_default()
    }

    /// Check if anything is currently scheduled.
    pub fn is_empty(&self) -> bool {
        self.scheduled.is_empty()
    }

    /// Get an iterator over all scheduled validators.
    pub fn iter(&self) -> impl Iterator<Item = (&IdentityKey, &Unbonding)> {
        self.scheduled.iter()
    }
}

impl IntoIterator for Scheduled {
    type Item = <BTreeMap<IdentityKey, Unbonding> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<IdentityKey, Unbonding> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.scheduled.into_iter()
    }
}

impl<'a> IntoIterator for &'a Scheduled {
    type Item = <&'a BTreeMap<IdentityKey, Unbonding> as IntoIterator>::Item;
    type IntoIter = <&'a BTreeMap<IdentityKey, Unbonding> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.scheduled.iter()
    }
}

impl Extend<(IdentityKey, Unbonding)> for Scheduled {
    fn extend<T: IntoIterator<Item = (IdentityKey, Unbonding)>>(&mut self, iter: T) {
        for (identity_key, unbonding) in iter {
            let entry = self.scheduled.entry(identity_key).or_default();
            entry.extend(unbonding.note_payloads);
            entry.extend(unbonding.nullifiers);
        }
    }
}

impl FromIterator<(IdentityKey, Unbonding)> for Scheduled {
    fn from_iter<T: IntoIterator<Item = (IdentityKey, Unbonding)>>(iter: T) -> Self {
        let mut scheduled = Scheduled::default();
        scheduled.extend(iter);
        scheduled
    }
}

impl Protobuf<pb::quarantined::Scheduled> for Scheduled {}

impl TryFrom<pb::quarantined::Scheduled> for Scheduled {
    type Error = anyhow::Error;

    fn try_from(value: pb::quarantined::Scheduled) -> Result<Self, Self::Error> {
        Ok(Self {
            scheduled: value
                .per_validator
                .into_iter()
                .map(
                    |pb::quarantined::ValidatorEntry {
                         identity_key,
                         unbonding,
                     }| {
                        if let Some(identity_key) = identity_key {
                            Ok((
                                identity_key.try_into()?,
                                unbonding
                                    .map(TryInto::try_into)
                                    .transpose()?
                                    .unwrap_or_default(),
                            ))
                        } else {
                            Err(anyhow::anyhow!("missing identity key"))
                        }
                    },
                )
                .collect::<Result<BTreeMap<_, _>, _>>()?,
        })
    }
}

impl From<Scheduled> for pb::quarantined::Scheduled {
    fn from(value: Scheduled) -> Self {
        Self {
            per_validator: value
                .scheduled
                .into_iter()
                .map(
                    |(identity_key, unbonding)| pb::quarantined::ValidatorEntry {
                        identity_key: Some(identity_key.into()),
                        unbonding: if unbonding.is_empty() {
                            None
                        } else {
                            Some(unbonding.into())
                        },
                    },
                )
                .collect(),
        }
    }
}

/// All the things unbonding from a specific validator (not specified here) in a specific (not
/// specified here) epoch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(
    into = "pb::quarantined::Unbonding",
    try_from = "pb::quarantined::Unbonding"
)]
pub struct Unbonding {
    pub note_payloads: Vec<AnnotatedNotePayload>,
    pub nullifiers: Vec<Nullifier>,
}

impl Unbonding {
    /// Add a new note.
    pub fn schedule_note(&mut self, note_payload: AnnotatedNotePayload) {
        self.note_payloads.push(note_payload)
    }

    /// Add a new nullifier.
    pub fn schedule_nullifier(&mut self, nullifier: Nullifier) {
        self.nullifiers.push(nullifier)
    }

    /// Check if anything is currently unbonding.
    pub fn is_empty(&self) -> bool {
        self.note_payloads.is_empty() && self.nullifiers.is_empty()
    }
}

impl Extend<AnnotatedNotePayload> for Unbonding {
    fn extend<T: IntoIterator<Item = AnnotatedNotePayload>>(&mut self, iter: T) {
        self.note_payloads.extend(iter);
    }
}

impl Extend<Nullifier> for Unbonding {
    fn extend<T: IntoIterator<Item = Nullifier>>(&mut self, iter: T) {
        self.nullifiers.extend(iter);
    }
}

impl FromIterator<AnnotatedNotePayload> for Unbonding {
    fn from_iter<T: IntoIterator<Item = AnnotatedNotePayload>>(iter: T) -> Self {
        let mut unbonding = Unbonding::default();
        unbonding.extend(iter);
        unbonding
    }
}

impl FromIterator<Nullifier> for Unbonding {
    fn from_iter<T: IntoIterator<Item = Nullifier>>(iter: T) -> Self {
        let mut unbonding = Unbonding::default();
        unbonding.extend(iter);
        unbonding
    }
}

impl Protobuf<pb::quarantined::Unbonding> for Unbonding {}

impl TryFrom<pb::quarantined::Unbonding> for Unbonding {
    type Error = anyhow::Error;

    fn try_from(value: pb::quarantined::Unbonding) -> Result<Self, Self::Error> {
        Ok(Self {
            note_payloads: value
                .note_payloads
                .into_iter()
                .map(AnnotatedNotePayload::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            nullifiers: value
                .nullifiers
                .into_iter()
                .map(Nullifier::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<Unbonding> for pb::quarantined::Unbonding {
    fn from(value: Unbonding) -> Self {
        Self {
            note_payloads: value.note_payloads.into_iter().map(Into::into).collect(),
            nullifiers: value.nullifiers.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Slashed {
    pub validators: Vec<IdentityKey>,
}

impl Protobuf<pb::Slashed> for Slashed {}

impl TryFrom<pb::Slashed> for Slashed {
    type Error = anyhow::Error;

    fn try_from(value: pb::Slashed) -> Result<Self, Self::Error> {
        Ok(Self {
            validators: value
                .validators
                .into_iter()
                .map(IdentityKey::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<Slashed> for pb::Slashed {
    fn from(value: Slashed) -> Self {
        Self {
            validators: value.validators.into_iter().map(Into::into).collect(),
        }
    }
}
