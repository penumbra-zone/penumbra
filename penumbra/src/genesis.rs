use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use penumbra_crypto::Note;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(
    try_from = "helpers::GenesisNotesHelper",
    into = "helpers::GenesisNotesHelper"
)]
pub struct GenesisNotes {
    notes: Vec<Note>,
}

mod helpers {
    use decaf377::{FieldExt, Fq};
    use penumbra_crypto::{asset, ka, keys::Diversifier, Value};

    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct GenesisNotesHelper {
        notes: Vec<NoteHelper>,
    }

    impl From<GenesisNotes> for GenesisNotesHelper {
        fn from(notes: GenesisNotes) -> Self {
            Self {
                notes: notes.notes.into_iter().map(From::from).collect(),
            }
        }
    }

    impl TryFrom<GenesisNotesHelper> for GenesisNotes {
        type Error = anyhow::Error;

        fn try_from(helper: GenesisNotesHelper) -> Result<Self, Self::Error> {
            Ok(Self {
                notes: helper
                    .notes
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
    }

    #[serde_as]
    #[derive(Deserialize, Serialize)]
    pub struct NoteHelper {
        #[serde_as(as = "serde_with::hex::Hex")]
        diversifier: [u8; 11],
        amount: u64,
        #[serde_as(as = "serde_with::hex::Hex")]
        note_blinding: [u8; 32],
        #[serde_as(as = "serde_with::hex::Hex")]
        asset_id: [u8; 32],
        #[serde_as(as = "serde_with::hex::Hex")]
        transmission_key: [u8; 32],
    }

    impl From<Note> for NoteHelper {
        fn from(note: Note) -> Self {
            Self {
                diversifier: note.diversifier().0,
                amount: note.value().amount,
                note_blinding: note.note_blinding().to_bytes(),
                asset_id: note.value().asset_id.to_bytes(),
                transmission_key: note.transmission_key().0,
            }
        }
    }

    impl TryFrom<NoteHelper> for Note {
        type Error = anyhow::Error;

        fn try_from(helper: NoteHelper) -> Result<Self, Self::Error> {
            let amount = helper.amount;
            let asset_id = asset::Id(Fq::from_bytes(helper.asset_id)?);
            let note_blinding = Fq::from_bytes(helper.note_blinding)?;
            let transmission_key = ka::Public(helper.transmission_key);
            let diversifier = Diversifier(helper.diversifier);

            let note = Note::new(
                diversifier,
                transmission_key,
                Value { asset_id, amount },
                note_blinding,
            )?;

            Ok(note)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ff::UniformRand;
    use penumbra_crypto::{keys::SpendKey, Fq, Note, Value};
    use rand_core::OsRng;

    #[test]
    fn genesis_notes_json() {
        let sk = SpendKey::generate(OsRng);
        let (dest0, _) = sk
            .full_viewing_key()
            .incoming()
            .payment_address(0u64.into());
        let (dest1, _) = sk
            .full_viewing_key()
            .incoming()
            .payment_address(1u64.into());
        let (dest2, _) = sk
            .full_viewing_key()
            .incoming()
            .payment_address(2u64.into());

        let value0 = Value {
            amount: 100,
            asset_id: b"pen".as_ref().into(),
        };
        let value1 = Value {
            amount: 1,
            asset_id: b"tungsten_cube".as_ref().into(),
        };
        let value2 = Value {
            amount: 1000,
            asset_id: b"pen".as_ref().into(),
        };

        let note0 = Note::new(
            *dest0.diversifier(),
            *dest0.transmission_key(),
            value0,
            Fq::rand(&mut OsRng),
        )
        .unwrap();
        let note1 = Note::new(
            *dest1.diversifier(),
            *dest1.transmission_key(),
            value1,
            Fq::rand(&mut OsRng),
        )
        .unwrap();
        let note2 = Note::new(
            *dest2.diversifier(),
            *dest2.transmission_key(),
            value2,
            Fq::rand(&mut OsRng),
        )
        .unwrap();

        let genesis_notes = GenesisNotes {
            notes: vec![note0, note1, note2],
        };

        let serialized = serde_json::to_string_pretty(&genesis_notes).unwrap();

        println!("\n{}\n", serialized);

        let genesis_notes2: GenesisNotes = serde_json::from_str(&serialized).unwrap();

        assert_eq!(genesis_notes, genesis_notes2);
    }
}
