use penumbra_chain::NoteSource;
use penumbra_crypto::Nullifier;
use tendermint::abci::{Event, EventAttributeIndexExt};

pub fn spend(nullifier: Nullifier, source: NoteSource) -> Event {
    Event::new(
        "spend",
        vec![
            ("nullifier", nullifier.to_string()).index(),
            ("source", source.to_string()).index(),
        ],
    )
}
