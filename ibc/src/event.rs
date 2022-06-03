use ibc::core::{
    ics02_client::{client_state::AnyClientState, header::AnyHeader, header::Header},
    ics24_host::identifier::ClientId,
};
use tendermint::abci::{Event, EventAttributeIndexExt};

pub fn create_client(client_id: ClientId, client_state: AnyClientState) -> Event {
    Event::new(
        "create_client",
        vec![
            ("client_id", client_id.to_string()).index(),
            ("client_type", client_state.client_type().to_string()).index(),
            ("consensus_height", client_state.latest_height().to_string()).index(),
        ],
    )
}

pub fn update_client(
    client_id: ClientId,
    client_state: AnyClientState,
    header: AnyHeader,
) -> Event {
    Event::new(
        "update_client",
        vec![
            ("client_id", client_id.to_string()).index(),
            ("client_type", client_state.client_type().to_string()).index(),
            ("consensus_height", header.height().to_string()).index(),
            ("header", header.encode_to_string()).index(),
        ],
    )
}
