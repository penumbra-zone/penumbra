use super::*;

/// The abstract description of an action performed by the wallet user.
#[derive(derivative::Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub struct ActionDescription(pub(super) action::Inner);

impl ActionDescription {
    /// Create a new send action.
    pub fn send(dest_address: Address, value: Value, memo: String) -> ActionDescription {
        Self(action::Inner::Send {
            value,
            dest_address: Box::new(dest_address),
            memo,
        })
    }

    /// Create a new fee action.
    pub fn fee(fee: u64) -> ActionDescription {
        Self(action::Inner::Fee { amount: fee })
    }

    /// Create a new delegate action.
    pub fn delegate(rate_data: RateData, unbonded_amount: u64) -> ActionDescription {
        Self(action::Inner::DelegateOrUndelegate {
            flow: action::DelegateFlow::Delegate { unbonded_amount },
            rate_data,
        })
    }

    /// Create a new undelegate action.
    pub fn undelegate(rate_data: RateData, delegation_amount: u64) -> ActionDescription {
        Self(action::Inner::DelegateOrUndelegate {
            flow: action::DelegateFlow::Undelegate { delegation_amount },
            rate_data,
        })
    }
}

/// The abstract description of an action performed by the wallet user (internal enum).
#[derive(Debug, Clone)]
pub(super) enum Inner {
    Send {
        dest_address: Box<Address>,
        value: Value,
        memo: String,
    },
    Fee {
        amount: u64,
    },
    DelegateOrUndelegate {
        flow: DelegateFlow,
        rate_data: RateData,
    },
}

/// An amount of a delegation or undelegation, tagged by which it is.
#[derive(Debug, Clone, Copy)]
pub(super) enum DelegateFlow {
    Delegate { unbonded_amount: u64 },
    Undelegate { delegation_amount: u64 },
}

impl DelegateFlow {
    pub fn amount(&self) -> u64 {
        match self {
            DelegateFlow::Delegate { unbonded_amount } => *unbonded_amount,
            DelegateFlow::Undelegate { delegation_amount } => *delegation_amount,
        }
    }

    pub fn is_delegate(&self) -> bool {
        matches!(self, DelegateFlow::Delegate { .. })
    }
}
