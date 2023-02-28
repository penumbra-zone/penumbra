use std::fmt::Display;

use anyhow::Result;

use penumbra_crypto::stake::Penalty;

use super::{ChainParameters, Ratio};

// The checks below validate that a parameter change is valid, since some parameter settings or
// combinations are nonsensical and should be rejected outright, regardless of governance.

#[deny(unused)] // We want to be really careful here to not examine fields!
impl ChainParameters {
    pub fn check_valid_update(&self, new: &ChainParameters) -> Result<()> {
        new.check_valid()?;

        let ChainParameters {
            chain_id,
            epoch_duration: _,
            unbonding_epochs: _,
            active_validator_limit,
            base_reward_rate: _,
            slashing_penalty_misbehavior: _,
            slashing_penalty_downtime: _,
            signed_blocks_window_len,
            missed_blocks_maximum: _,
            ibc_enabled: _,
            inbound_ics20_transfers_enabled: _,
            outbound_ics20_transfers_enabled: _,
            proposal_voting_blocks: _,
            proposal_deposit_amount: _,
            proposal_valid_quorum,
            proposal_pass_threshold,
            proposal_veto_threshold,
            // IMPORTANT: Don't use `..` here! We want to ensure every single field is verified!
        } = self;

        // Ensure that certain parameters are not changed by the update:
        check_invariant([(chain_id, &new.chain_id, "chain ID")])?;
        check_invariant([
            (
                active_validator_limit,
                &new.active_validator_limit,
                "active validator limit",
            ),
            (
                signed_blocks_window_len,
                &new.signed_blocks_window_len,
                "signed blocks window length",
            ),
        ])?;
        check_invariant([
            (
                proposal_valid_quorum,
                &new.proposal_valid_quorum,
                "proposal valid quorum",
            ),
            (
                proposal_pass_threshold,
                &new.proposal_pass_threshold,
                "proposal pass threshold",
            ),
            (
                proposal_veto_threshold,
                &new.proposal_veto_threshold,
                "proposal veto threshold",
            ),
        ])?;

        Ok(())
    }

    pub fn check_valid(&self) -> Result<()> {
        let ChainParameters {
            chain_id,
            epoch_duration,
            unbonding_epochs,
            active_validator_limit,
            base_reward_rate,
            slashing_penalty_misbehavior,
            slashing_penalty_downtime,
            signed_blocks_window_len,
            missed_blocks_maximum,
            ibc_enabled,
            inbound_ics20_transfers_enabled,
            outbound_ics20_transfers_enabled,
            proposal_voting_blocks,
            proposal_deposit_amount,
            proposal_valid_quorum,
            proposal_pass_threshold,
            proposal_veto_threshold,
            // IMPORTANT: Don't use `..` here! We want to ensure every single field is verified!
        } = self;

        check_all([
            (!chain_id.is_empty(), "chain ID must be a non-empty string"),
            (
                *epoch_duration >= 1,
                "epoch duration must be at least one block",
            ),
            (
                *unbonding_epochs >= 1,
                "unbonding must take at least one epoch",
            ),
            (
                *active_validator_limit > 3,
                "active validator limit must be at least 4",
            ),
            (
                *base_reward_rate >= 1,
                "base reward rate must be at least 1 basis point",
            ),
            (
                *slashing_penalty_misbehavior >= Penalty(1),
                "slashing penalty (misbehavior) must be at least 1 basis point",
            ),
            (
                *slashing_penalty_misbehavior <= Penalty(10_000),
                "slashing penalty (misbehavior) must be at most 10,000 basis points",
            ),
            (
                *slashing_penalty_downtime >= Penalty(1),
                "slashing penalty (downtime) must be at least 1 basis point",
            ),
            (
                *slashing_penalty_downtime <= Penalty(10_000),
                "slashing penalty (downtime) must be at most 10,000 basis points",
            ),
            (
                *signed_blocks_window_len >= 2,
                "signed blocks window length must be at least 2",
            ),
            (
                *missed_blocks_maximum >= 1,
                "missed blocks maximum must be at least 1",
            ),
            (
                (!*inbound_ics20_transfers_enabled && !*outbound_ics20_transfers_enabled)
                    || *ibc_enabled,
                "IBC must be enabled if either inbound or outbound ICS20 transfers are enabled",
            ),
            (
                *proposal_voting_blocks >= 1,
                "proposal voting blocks must be at least 1",
            ),
            (
                *proposal_deposit_amount >= 1u64.into(),
                "proposal deposit amount must be at least 1",
            ),
            (
                *proposal_valid_quorum > Ratio::new(0, 1),
                "proposal valid quorum must be greater than 0",
            ),
            (
                *proposal_pass_threshold > Ratio::new(1, 2),
                "proposal pass threshold must be greater than 1/2",
            ),
            (
                *proposal_veto_threshold > Ratio::new(1, 2),
                "proposal veto threshold must be greater than 1/2",
            ),
        ])
    }
}

/// Ensure all of the booleans are true, and if any are false, generate an error describing which
/// failed, based on the provided descriptions.
fn check_all<'a>(checks: impl IntoIterator<Item = (bool, impl Display + 'a)>) -> Result<()> {
    let failed_because = checks
        .into_iter()
        .filter_map(|(ok, description)| {
            if !ok {
                Some(description.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if !failed_because.is_empty() {
        anyhow::bail!("invalid chain parameters: {}", failed_because.join(", "));
    }

    Ok(())
}

/// Ensure that all of the provided pairs of values are equal, and if any are not, generate an error
/// stating that the varying names can't be changed.
fn check_invariant<'a, T: Eq + 'a>(
    sides: impl IntoIterator<Item = (&'a T, &'a T, impl Display + 'a)>,
) -> Result<()> {
    check_all(
        sides
            .into_iter()
            .map(|(old, new, name)| ((*old == *new), format!("{name} can't be changed"))),
    )
}
