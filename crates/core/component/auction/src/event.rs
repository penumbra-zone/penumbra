use tendermint::abci::{Event, EventAttributeIndexExt};

pub fn stub_event(delegate: &Delegate) -> Event {
    Event::new("stub_event", [])
}
