use ibc::core::ics24_host::identifier::ConnectionId;
use jmt::KeyHash;

pub fn connection(commitment_prefix: &str, connection_id: &ConnectionId) -> KeyHash {
    format!(
        "{}/connections/{}",
        commitment_prefix,
        connection_id.as_str()
    )
    .into()
}

pub fn connection_counter() -> KeyHash {
    format!("ibc/ics03-connection/connection_counter").into()
}
