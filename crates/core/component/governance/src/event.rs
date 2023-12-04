use tendermint::abci::{Event, EventAttributeIndexExt};

use crate::ProposalPayload;

pub fn proposal_payload(proposal_id: u64, payload: &ProposalPayload) -> Event {
    match payload {
        ProposalPayload::Signaling { commit } => todo!(),
        ProposalPayload::Emergency { halt_chain } => todo!(),
        ProposalPayload::ParameterChange { old, new } => todo!(),
        ProposalPayload::DaoSpend { transaction_plan } => todo!(),
        ProposalPayload::UpgradePlan { height } => todo!(),
        ProposalPayload::UnplannedIbcUpgrade {
            connection_id,
            new_config,
        } => Event::new(
            "proposal_unplanned_ibc_upgrade",
            [
                ("proposal_id", unplanned_upgrade.proposal_id.to_string()).index(),
                (
                    "ibc_channel_id",
                    unplanned_upgrade.ibc_channel_id.to_string(),
                )
                    .index(),
                (
                    "ibc_packet_data",
                    unplanned_upgrade.ibc_packet_data.to_string(),
                )
                    .index(),
            ],
        ),
        ProposalPayload::FreezeIbcClient { client_id } => todo!(),
        ProposalPayload::UnfreezeIbcClient { client_id } => todo!(),
    }
}

pub fn undelegate(undelegate: &Undelegate) -> Event {
    Event::new(
        "action_undelegate",
        [
            ("validator", undelegate.validator_identity.to_string()).index(),
            ("amount", undelegate.unbonded_amount.to_string()).no_index(),
        ],
    )
}
