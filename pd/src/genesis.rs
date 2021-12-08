use std::str::FromStr;

use ark_ff::UniformRand;
use decaf377::{FieldExt, Fq};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use penumbra_crypto::{asset, ka, keys::Diversifier, note, Address, Note, Value};

pub fn generate_genesis_notes(
    rng: &mut ChaCha20Rng,
    genesis_allocations: Vec<GenesisAddr>,
) -> Vec<GenesisNote> {
    let mut notes = Vec::<GenesisNote>::new();
    for genesis_addr in genesis_allocations {
        let note = GenesisNote::new(
            *genesis_addr.address.diversifier(),
            *genesis_addr.address.transmission_key(),
            GenesisValue {
                amount: genesis_addr.amount,
                asset_denom: genesis_addr.denom,
            },
            Fq::rand(rng),
        )
        .expect("note created successfully");
        notes.push(note);
    }
    notes
}

/// Value used only during genesis. The difference from [`Value`] is that
/// the human-readable asset denomination is stored rather than the hashed
/// asset ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisValue {
    pub amount: u64,
    /// The asset's human-readable denomination.
    pub asset_denom: String,
}

#[derive(Debug)]
pub struct GenesisAddr {
    pub amount: u64,
    pub denom: String,
    pub address: Address,
}

impl FromStr for GenesisAddr {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fields: Vec<&str> = s
            .trim_matches(|p| p == '(' || p == ')')
            .split(',')
            .collect();

        let amount_fromstr = fields[0].parse::<u64>()?;
        let denom_fromstr = fields[1].trim().to_string();
        let address_fromstr = Address::from_str(fields[2].trim())?;

        Ok(GenesisAddr {
            amount: amount_fromstr,
            denom: denom_fromstr,
            address: address_fromstr,
        })
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct GenesisAppState {
    /// Initial notes
    pub notes: Vec<GenesisNote>,
    /// Epoch duration in terms of blocks
    pub epoch_duration: u64,
}

#[serde_as]
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct GenesisNote {
    #[serde_as(as = "serde_with::hex::Hex")]
    pub diversifier: [u8; 11],
    pub amount: u64,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub note_blinding: [u8; 32],
    pub asset_denom: String,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub transmission_key: [u8; 32],
}

impl GenesisNote {
    pub fn new(
        diversifier: Diversifier,
        transmission_key: ka::Public,
        value: GenesisValue,
        note_blinding: Fq,
    ) -> Result<Self, note::Error> {
        Ok(GenesisNote {
            diversifier: diversifier.0,
            amount: value.amount,
            note_blinding: note_blinding.to_bytes(),
            asset_denom: value.asset_denom,
            transmission_key: transmission_key.0,
        })
    }
}

impl TryFrom<&GenesisNote> for Note {
    type Error = anyhow::Error;

    fn try_from(genesis_note: &GenesisNote) -> Result<Self, Self::Error> {
        let amount = genesis_note.amount;
        let asset_denom = &genesis_note.asset_denom;
        let note_blinding = Fq::from_bytes(genesis_note.note_blinding)?;
        let transmission_key = ka::Public(genesis_note.transmission_key);
        let diversifier = Diversifier(genesis_note.diversifier);

        let note = Note::from_parts(
            diversifier,
            transmission_key,
            Value {
                amount,
                asset_id: asset::Denom(asset_denom.clone()).into(),
            },
            note_blinding,
        )?;

        Ok(note)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use penumbra_crypto::keys::SpendKey;
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

        let value0 = GenesisValue {
            amount: 100,
            asset_denom: "penumbra".to_string(),
        };
        let value1 = GenesisValue {
            amount: 1,
            asset_denom: "tungsten_cube".to_string(),
        };
        let value2 = GenesisValue {
            amount: 1000,
            asset_denom: "penumbra".to_string(),
        };

        let note0 = GenesisNote::new(
            *dest0.diversifier(),
            *dest0.transmission_key(),
            value0,
            Fq::rand(&mut OsRng),
        )
        .unwrap();
        let note1 = GenesisNote::new(
            *dest1.diversifier(),
            *dest1.transmission_key(),
            value1,
            Fq::rand(&mut OsRng),
        )
        .unwrap();
        let note2 = GenesisNote::new(
            *dest2.diversifier(),
            *dest2.transmission_key(),
            value2,
            Fq::rand(&mut OsRng),
        )
        .unwrap();

        let genesis_notes = vec![note0, note1, note2];

        let serialized = serde_json::to_string_pretty(&genesis_notes).unwrap();

        println!("\n{}\n", serialized);

        let genesis_notes2: Vec<GenesisNote> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(genesis_notes, genesis_notes2);
    }
}
