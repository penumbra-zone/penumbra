use anyhow::Result;
use tendermint::abci;
use prost::{Message, Name};


pub trait ProtoEvent: Message + Name + Sized {
    fn into_event(&self) -> abci::Event {
        unimplemented!();
    }

    fn from_event(event: &abci::Event) -> Result<Self> {
        unimplemented!()
    }
}