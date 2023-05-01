use penumbra_crypto::Nullifier;
use tendermint::abci::{Event, EventAttributeIndexExt};

pub fn spend(nullifier: &Nullifier) -> Event {
    Event::new("spend", [("nullifier", nullifier.to_string()).index()])
}

/*
pub fn state_payload(payload: &StatePayload) -> Event {
    Event::new(
        "state_payload",
        [("commitment", payload.commitment().to_string()).index()],
    )
}
 */
