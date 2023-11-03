use crate::component::app_handler::AppHandler;
use crate::IbcAction;
use std::marker::PhantomData;

pub struct IbcActionWithHandler<H>(IbcAction, PhantomData<H>);

impl<H: AppHandler> IbcActionWithHandler<H> {
    pub fn new(action: IbcAction) -> Self {
        Self(action, PhantomData)
    }

    pub fn action(&self) -> &IbcAction {
        &self.0
    }

    pub fn into_inner(self) -> IbcAction {
        self.0
    }
}

impl<H: AppHandler> From<IbcActionWithHandler<H>> for IbcAction {
    fn from(value: IbcActionWithHandler<H>) -> Self {
        value.0
    }
}

impl IbcAction {
    pub fn with_handler<H: AppHandler>(self) -> IbcActionWithHandler<H> {
        IbcActionWithHandler::new(self)
    }
}
