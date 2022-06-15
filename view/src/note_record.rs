use penumbra_crypto::{
    asset,
    ka::Public,
    keys::{Diversifier, DiversifierIndex},
    note, FieldExt, Fq, IdentityKey, Note, Nullifier, Value,
};
use penumbra_proto::{view as pb, Protobuf};
use penumbra_tct as tct;

use serde::{Deserialize, Serialize};
use sqlx::Row;

/// Corresponds to the NoteRecord proto
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "pb::NoteRecord", into = "pb::NoteRecord")]
pub struct NoteRecord {
    pub note_commitment: note::Commitment,
    pub note: Note,
    pub diversifier_index: DiversifierIndex,
    pub height_created: u64,
    pub status: Status,
}

impl NoteRecord {
    /// Returns the position of the note, or `None` if it is quarantined.
    pub fn position(&self) -> Option<tct::Position> {
        match self.status {
            Status::Applied { position, .. } => Some(position),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    Quarantined {
        unbonding_epoch: u64,
        identity_key: IdentityKey,
    },
    Applied {
        nullifier: Nullifier,
        position: tct::Position,
        height_spent: Option<(u64, SpendStatus)>,
    },
}

impl Status {
    #[allow(clippy::type_complexity)]
    pub fn into_parts(
        self,
    ) -> (
        Option<u64>,
        Option<Nullifier>,
        Option<tct::Position>,
        Option<IdentityKey>,
        Option<u64>,
    ) {
        match self {
            Status::Quarantined {
                identity_key,
                unbonding_epoch,
            } => (None, None, None, Some(identity_key), Some(unbonding_epoch)),
            Status::Applied {
                height_spent,
                nullifier,
                position,
            } => {
                let (height_spent, (unbonding_epoch, identity_key)) =
                    if let Some((height_spent, spend_status)) = height_spent {
                        (
                            Some(height_spent),
                            match spend_status {
                                SpendStatus::Committed => (None, None),
                                SpendStatus::Quarantined {
                                    unbonding_epoch,
                                    identity_key,
                                } => (Some(unbonding_epoch), Some(identity_key)),
                            },
                        )
                    } else {
                        (None, (None, None))
                    };
                (
                    height_spent,
                    Some(nullifier),
                    Some(position),
                    identity_key,
                    unbonding_epoch,
                )
            }
        }
    }

    pub fn try_from_parts(
        height_spent: Option<u64>,
        nullifier: Option<Nullifier>,
        position: Option<tct::Position>,
        identity_key: Option<IdentityKey>,
        unbonding_epoch: Option<u64>,
    ) -> anyhow::Result<Self> {
        Ok(if let Some(height_spent) = height_spent {
            Self::Applied {
                nullifier: nullifier.ok_or_else(|| anyhow::anyhow!("missing nullifier"))?,
                position: position.ok_or_else(|| anyhow::anyhow!("missing position"))?,
                height_spent: Some((
                    height_spent,
                    if let Some(unbonding_epoch) = unbonding_epoch {
                        SpendStatus::Quarantined {
                            unbonding_epoch,
                            identity_key: identity_key
                                .ok_or_else(|| anyhow::anyhow!("missing identity key"))?,
                        }
                    } else {
                        SpendStatus::Committed
                    },
                )),
            }
        } else if let Some(unbonding_epoch) = unbonding_epoch {
            if nullifier.is_some() {
                return Err(anyhow::anyhow!(
                    "nullifier should not be present for quarantined notes"
                ));
            }
            if position.is_some() {
                return Err(anyhow::anyhow!(
                    "position should not be present for quarantined notes"
                ));
            }
            Self::Quarantined {
                unbonding_epoch,
                identity_key: identity_key
                    .ok_or_else(|| anyhow::anyhow!("missing identity_key"))?,
            }
        } else {
            if identity_key.is_some() {
                return Err(anyhow::anyhow!(
                    "identity_key should not be present for unspent, unquarantined notes"
                ));
            }
            if unbonding_epoch.is_some() {
                return Err(anyhow::anyhow!(
                    "unbonding epoch should not be present for unspent, unquarantined notes"
                ));
            }
            Self::Applied {
                nullifier: nullifier.ok_or_else(|| anyhow::anyhow!("missing nullifier"))?,
                position: position.ok_or_else(|| anyhow::anyhow!("missing position"))?,
                height_spent: None,
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpendStatus {
    Committed,
    Quarantined {
        unbonding_epoch: u64,
        identity_key: IdentityKey,
    },
}

impl Status {
    /// Indicates that the note has been applied, i.e. it is not quarantined.
    pub fn is_applied(&self) -> bool {
        matches!(self, Status::Applied { .. })
    }

    /// Indicates that the note is quarantined.
    pub fn is_quarantined(&self) -> bool {
        matches!(self, Status::Quarantined { .. })
    }
}

impl Protobuf<pb::NoteRecord> for NoteRecord {}

impl From<NoteRecord> for pb::NoteRecord {
    fn from(v: NoteRecord) -> Self {
        pb::NoteRecord {
            note_commitment: Some(v.note_commitment.into()),
            note: Some(v.note.into()),
            diversifier_index: Some(v.diversifier_index.into()),
            height_created: v.height_created,
            status: Some(match v.status {
                Status::Quarantined {
                    unbonding_epoch,
                    identity_key,
                } => pb::note_record::Status::Quarantined(pb::note_record::Quarantined {
                    unbonding_epoch,
                    identity_key: Some(identity_key.into()),
                }),
                Status::Applied {
                    nullifier,
                    position,
                    height_spent,
                } => pb::note_record::Status::Applied(pb::note_record::Applied {
                    nullifier: Some(nullifier.into()),
                    position: position.into(),
                    height_spent: height_spent.map(|p| p.0),
                    spend_status: height_spent.and_then(|p| {
                        if let SpendStatus::Quarantined {
                            identity_key,
                            unbonding_epoch,
                        } = p.1
                        {
                            Some(pb::note_record::Quarantined {
                                identity_key: Some(identity_key.into()),
                                unbonding_epoch,
                            })
                        } else {
                            None
                        }
                    }),
                }),
            }),
        }
    }
}

impl TryFrom<pb::NoteRecord> for NoteRecord {
    type Error = anyhow::Error;
    fn try_from(v: pb::NoteRecord) -> Result<Self, Self::Error> {
        Ok(NoteRecord {
            note_commitment: v
                .note_commitment
                .ok_or_else(|| anyhow::anyhow!("missing note commitment"))?
                .try_into()?,
            note: v
                .note
                .ok_or_else(|| anyhow::anyhow!("missing note"))?
                .try_into()?,
            diversifier_index: v
                .diversifier_index
                .ok_or_else(|| anyhow::anyhow!("missing diversifier index"))?
                .try_into()?,
            height_created: v.height_created,
            status: if let Some(status) = v.status {
                match status {
                    pb::note_record::Status::Quarantined(v) => Status::Quarantined {
                        unbonding_epoch: v.unbonding_epoch,
                        identity_key: v
                            .identity_key
                            .ok_or_else(|| anyhow::anyhow!("missing identity key"))?
                            .try_into()?,
                    },
                    pb::note_record::Status::Applied(v) => Status::Applied {
                        nullifier: v
                            .nullifier
                            .ok_or_else(|| anyhow::anyhow!("missing nullifier"))?
                            .try_into()?,
                        position: v.position.try_into()?,
                        height_spent: if let Some(height_spent) = v.height_spent {
                            Some((
                                height_spent,
                                v.spend_status
                                    .map(
                                        |pb::note_record::Quarantined {
                                             identity_key,
                                             unbonding_epoch,
                                         }| {
                                            Ok::<_, anyhow::Error>(SpendStatus::Quarantined {
                                                identity_key: identity_key
                                                    .map(TryInto::try_into)
                                                    .transpose()?
                                                    .ok_or_else(|| {
                                                        anyhow::anyhow!("missing identity key")
                                                    })?,
                                                unbonding_epoch,
                                            })
                                        },
                                    )
                                    .transpose()?
                                    .unwrap_or(SpendStatus::Committed),
                            ))
                        } else if v.spend_status.is_some() {
                            return Err(anyhow::anyhow!(
                                "spend status present but height spent is not"
                            ));
                        } else {
                            None
                        },
                    },
                }
            } else {
                return Err(anyhow::anyhow!("missing status"));
            },
        })
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for NoteRecord {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        // This is not a fun time.
        // Mostly on account of sqlx::Error.

        let diversifier =
            Diversifier::try_from(row.get::<'r, &[u8], _>("diversifier")).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "diversifier".to_string(),
                    source: e.into(),
                }
            })?;

        let diversifier_index = DiversifierIndex::try_from(
            row.get::<'r, &[u8], _>("diversifier_index"),
        )
        .map_err(|e| sqlx::Error::ColumnDecode {
            index: "diversifier_index".to_string(),
            source: e.into(),
        })?;

        let transmission_key = Public(
            <[u8; 32]>::try_from(row.get::<'r, &[u8], _>("transmission_key")).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "transmission_key".to_string(),
                    source: e.into(),
                }
            })?,
        );

        let amount = row.get::<'r, i64, _>("amount") as u64;

        let asset_id = asset::Id(
            Fq::from_bytes(
                <[u8; 32]>::try_from(row.get::<'r, &[u8], _>("asset_id")).map_err(|e| {
                    sqlx::Error::ColumnDecode {
                        index: "asset_id".to_string(),
                        source: e.into(),
                    }
                })?,
            )
            .map_err(|e| sqlx::Error::ColumnDecode {
                index: "asset_id".to_string(),
                source: e.into(),
            })?,
        );

        let note_blinding = Fq::from_bytes(
            <[u8; 32]>::try_from(row.get::<'r, &[u8], _>("blinding_factor")).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "blinding_factor".to_string(),
                    source: e.into(),
                }
            })?,
        )
        .map_err(|e| sqlx::Error::ColumnDecode {
            index: "blinding_factor".to_string(),
            source: e.into(),
        })?;

        let note_commitment = note::Commitment::try_from(
            row.get::<'r, &[u8], _>("note_commitment"),
        )
        .map_err(|e| sqlx::Error::ColumnDecode {
            index: "note_commitment".to_string(),
            source: e.into(),
        })?;

        let height_created = row.get::<'r, i64, _>("height_created") as u64;

        let value = Value { amount, asset_id };
        let note =
            Note::from_parts(diversifier, transmission_key, value, note_blinding).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "note".to_string(),
                    source: e.into(),
                }
            })?;

        let status = {
            // Pull all these fields out of the database as optional fields, since any one of them
            // could be missing for some statuses, then try to construct a status out of them, and
            // fail if the set of nulls vs. non-nulls is not valid
            let unbonding_epoch: Option<u64> = row
                .get::<'r, Option<i64>, _>("unbonding_epoch")
                .map(|i| i as u64);
            let identity_key: Option<IdentityKey> = row
                .get::<'r, Option<&[u8]>, _>("identity_key")
                .map(IdentityKey::decode)
                .transpose()
                .map_err(|e| sqlx::Error::ColumnDecode {
                    index: "identity_key".to_string(),
                    source: e.into(),
                })?;
            let nullifier: Option<Nullifier> = row
                .get::<'r, Option<&[u8]>, _>("nullifier")
                .map(Nullifier::decode)
                .transpose()
                .map_err(|e| sqlx::Error::ColumnDecode {
                    index: "nullifier".to_string(),
                    source: e.into(),
                })?;
            let position: Option<tct::Position> = row
                .get::<'r, Option<i64>, _>("position")
                .map(|i| (i as u64).try_into())
                .transpose()
                .map_err(
                    |e: <tct::Position as TryFrom<u64>>::Error| sqlx::Error::ColumnDecode {
                        index: "position".to_string(),
                        source: e.into(),
                    },
                )?;
            let height_spent: Option<u64> = row
                .get::<'r, Option<i64>, _>("height_spent")
                .map(|v| v as u64);

            Status::try_from_parts(
                height_spent,
                nullifier,
                position,
                identity_key,
                unbonding_epoch,
            )
            .map_err(|e| sqlx::Error::Decode(e.into()))?
        };

        Ok(NoteRecord {
            note_commitment,
            note,
            diversifier_index,
            height_created,
            status,
        })
    }
}
