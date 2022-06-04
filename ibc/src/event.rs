use ibc::core::{
    ics02_client::{client_state::AnyClientState, header::AnyHeader, header::Header},
    ics03_connection::connection::ConnectionEnd,
    ics03_connection::connection::Counterparty,
    ics24_host::identifier::ClientId,
    ics24_host::identifier::ConnectionId,
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

pub fn connection_open_init(
    connection_id: &ConnectionId,
    client_id: &ClientId,
    counterparty: &Counterparty,
) -> Event {
    Event::new(
        "connection_open_init",
        vec![
            ("connection_id", connection_id.to_string()).index(),
            ("client_id", client_id.to_string()).index(),
            (
                "counterparty_client_id",
                counterparty.client_id().to_string(),
            )
                .index(),
            (
                "counterparty_conneciton_id",
                counterparty.connection_id().unwrap().to_string(),
            )
                .index(),
        ],
    )
}

pub fn connection_open_try(
    connection_id: &ConnectionId,
    client_id: &ClientId,
    counterparty: &Counterparty,
) -> Event {
    Event::new(
        "connection_open_try",
        vec![
            ("connection_id", connection_id.to_string()).index(),
            ("client_id", client_id.to_string()).index(),
            (
                "counterparty_client_id",
                counterparty.client_id().to_string(),
            )
                .index(),
            (
                "counterparty_conneciton_id",
                counterparty.connection_id().unwrap().to_string(),
            )
                .index(),
        ],
    )
}

pub fn connection_open_ack(connection_id: &ConnectionId, connection_end: &ConnectionEnd) -> Event {
    Event::new(
        "connection_open_ack",
        vec![
            ("connection_id", connection_id.to_string()).index(),
            ("client_id", connection_end.client_id().to_string()).index(),
            (
                "counterparty_client_id",
                connection_end.counterparty().client_id().to_string(),
            )
                .index(),
            (
                "counterparty_connection_id",
                connection_end
                    .counterparty()
                    .connection_id()
                    .unwrap()
                    .to_string(),
            )
                .index(),
        ],
    )
}

pub fn connection_open_confirm(
    connection_id: &ConnectionId,
    connection_end: &ConnectionEnd,
) -> Event {
    Event::new(
        "connection_open_confirm",
        vec![
            ("connection_id", connection_id.to_string()).index(),
            ("client_id", connection_end.client_id().to_string()).index(),
            (
                "counterparty_client_id",
                connection_end.counterparty().client_id().to_string(),
            )
                .index(),
            (
                "counterparty_connection_id",
                connection_end
                    .counterparty()
                    .connection_id()
                    .unwrap()
                    .to_string(),
            )
                .index(),
        ],
    )
}
