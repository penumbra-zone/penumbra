use crate::component::app_handler::AppHandler;
use crate::IbcRelay;
use std::marker::PhantomData;

use super::HostInterface;

pub struct IbcRelayWithHandlers<AH, HI>(IbcRelay, PhantomData<AH>, PhantomData<HI>);

impl<AH, HI> IbcRelayWithHandlers<AH, HI> {
    pub fn new(action: IbcRelay) -> Self {
        Self(action, PhantomData, PhantomData)
    }

    pub fn action(&self) -> &IbcRelay {
        &self.0
    }

    pub fn into_inner(self) -> IbcRelay {
        self.0
    }
}

impl<AH, HI> From<IbcRelayWithHandlers<AH, HI>> for IbcRelay {
    fn from(value: IbcRelayWithHandlers<AH, HI>) -> Self {
        value.0
    }
}

impl IbcRelay {
    pub fn with_handler<AH: AppHandler, HI: HostInterface>(self) -> IbcRelayWithHandlers<AH, HI> {
        IbcRelayWithHandlers::new(self)
    }
}
