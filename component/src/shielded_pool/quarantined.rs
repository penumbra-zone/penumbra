use std::collections::BTreeMap;

use penumbra_crypto::{IdentityKey, NotePayload, Nullifier};
use penumbra_proto::{chain as pb, Protobuf};

/// All the things quarantined in a single epoch.
#[derive(Debug, Clone, Default)]
pub struct Quarantined {
    pub quarantined: BTreeMap<IdentityKey, PerValidator>,
}

impl Quarantined {
    pub fn insert_per_validator(&mut self, identity_key: IdentityKey, per_validator: PerValidator) {
        let existing = self.quarantined.entry(identity_key).or_default();
        existing.notes.extend(per_validator.notes);
        existing.nullifiers.extend(per_validator.nullifiers);
    }

    pub fn insert_note(&mut self, identity_key: IdentityKey, note_payload: NotePayload) {
        self.quarantined
            .entry(identity_key)
            .or_default()
            .insert_note(note_payload)
    }

    pub fn insert_nullifier(&mut self, identity_key: IdentityKey, nullifier: Nullifier) {
        self.quarantined
            .entry(identity_key)
            .or_default()
            .insert_nullifier(nullifier)
    }
}

impl IntoIterator for Quarantined {
    type Item = (IdentityKey, PerValidator);

    type IntoIter = <BTreeMap<IdentityKey, PerValidator> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.quarantined.into_iter()
    }
}

impl Extend<(IdentityKey, PerValidator)> for Quarantined {
    fn extend<T: IntoIterator<Item = (IdentityKey, PerValidator)>>(&mut self, iter: T) {
        for (identity_key, new) in iter {
            self.insert_per_validator(identity_key, new);
        }
    }
}

/// All the things quarantined for a given validator.
#[derive(Debug, Clone, Default)]
pub struct PerValidator {
    pub notes: Vec<NotePayload>,
    pub nullifiers: Vec<Nullifier>,
}

impl PerValidator {
    pub fn insert_note(&mut self, note_payload: NotePayload) {
        self.notes.push(note_payload);
    }

    pub fn insert_nullifier(&mut self, nullifier: Nullifier) {
        self.nullifiers.push(nullifier);
    }
}

impl Protobuf<pb::Quarantined> for Quarantined {}

impl TryFrom<pb::Quarantined> for Quarantined {
    type Error = anyhow::Error;

    fn try_from(value: pb::Quarantined) -> Result<Self, Self::Error> {
        let mut quarantined: BTreeMap<IdentityKey, PerValidator> = BTreeMap::new();
        for pb::QuarantinedPerValidator {
            identity_key,
            note_payloads,
            nullifiers,
        } in value.quarantined
        {
            let identity_key = identity_key
                .ok_or_else(|| anyhow::anyhow!("missing validator identity key"))?
                .try_into()?;
            let note_payloads = note_payloads
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<NotePayload>, _>>()?;
            let nullifiers = nullifiers
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Nullifier>, _>>()?;

            let per_validator = quarantined.entry(identity_key).or_default();
            per_validator.notes.extend(note_payloads);
            per_validator.nullifiers.extend(nullifiers);
        }
        Ok(Quarantined { quarantined })
    }
}

impl From<Quarantined> for pb::Quarantined {
    fn from(Quarantined { quarantined }: Quarantined) -> Self {
        let mut pb = pb::Quarantined {
            quarantined: Default::default(),
        };
        for (identity_key, per_validator) in quarantined.into_iter() {
            let identity_key = Some(identity_key.into());
            let note_payloads = per_validator.notes.into_iter().map(Into::into).collect();
            let nullifiers = per_validator
                .nullifiers
                .into_iter()
                .map(Into::into)
                .collect();

            pb.quarantined.push(pb::QuarantinedPerValidator {
                identity_key,
                note_payloads,
                nullifiers,
            });
        }
        pb
    }
}
