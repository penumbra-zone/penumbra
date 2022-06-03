use penumbra_crypto::Nullifier;
use tendermint::abci::{Event, EventAttributeIndexExt};

pub fn spend(nullifier: Nullifier) -> Event {
    Event::new("spend", vec![("nullifier", nullifier.to_string()).index()])
}

pub fn quarantine_spend(nullifier: Nullifier) -> Event {
    Event::new(
        "quarantine_spend",
        vec![("nullifier", nullifier.to_string()).index()],
    )
}
