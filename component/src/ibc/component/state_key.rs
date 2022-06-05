use ibc::core::ics24_host::identifier::ConnectionId;
use jmt::KeyHash;

use crate::ibc::COMMITMENT_PREFIX;

pub fn connection(connection_id: &ConnectionId) -> KeyHash {
    format!(
        "{}/connections/{}",
        COMMITMENT_PREFIX,
        connection_id.as_str()
    )
    .into()
}

pub fn connection_counter() -> KeyHash {
    format!("ibc/ics03-connection/connection_counter").into()
}
