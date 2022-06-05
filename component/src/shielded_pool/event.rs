use penumbra_crypto::Nullifier;
use tendermint::abci::{Event, EventAttributeIndexExt};

pub fn spend(nullifier: Nullifier) -> Event {
    Event::new("spend", vec![("nullifier", nullifier.to_string()).index()])
}
