use crate::Ics20Withdrawal;
use penumbra_sdk_ibc::component::HostInterface;
use std::marker::PhantomData;

pub struct Ics20WithdrawalWithHandler<HI>(Ics20Withdrawal, PhantomData<HI>);

impl<HI> Ics20WithdrawalWithHandler<HI> {
    pub fn new(action: Ics20Withdrawal) -> Self {
        Self(action, PhantomData)
    }

    pub fn action(&self) -> &Ics20Withdrawal {
        &self.0
    }

    pub fn into_inner(self) -> Ics20Withdrawal {
        self.0
    }
}

impl<HI> From<Ics20WithdrawalWithHandler<HI>> for Ics20Withdrawal {
    fn from(value: Ics20WithdrawalWithHandler<HI>) -> Self {
        value.0
    }
}

impl Ics20Withdrawal {
    pub fn with_handler<HI: HostInterface>(self) -> Ics20WithdrawalWithHandler<HI> {
        Ics20WithdrawalWithHandler::new(self)
    }
}
