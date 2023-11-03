use crate::component::app_handler::AppHandler;
use crate::IbcRelay;
use std::marker::PhantomData;

pub struct IbcActionWithHandler<H>(IbcRelay, PhantomData<H>);

impl<H: AppHandler> IbcActionWithHandler<H> {
    pub fn new(action: IbcRelay) -> Self {
        Self(action, PhantomData)
    }

    pub fn action(&self) -> &IbcRelay {
        &self.0
    }

    pub fn into_inner(self) -> IbcRelay {
        self.0
    }
}

impl<H: AppHandler> From<IbcActionWithHandler<H>> for IbcRelay {
    fn from(value: IbcActionWithHandler<H>) -> Self {
        value.0
    }
}

impl IbcRelay {
    pub fn with_handler<H: AppHandler>(self) -> IbcActionWithHandler<H> {
        IbcActionWithHandler::new(self)
    }
}
