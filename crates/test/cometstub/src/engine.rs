#![allow(dead_code)]

use tendermint::Block;

use crate::validator::Validator;

struct Engine<C> {
    app: C,
    last_app_hash: Vec<u8>
}

struct BlockBuilder<'e, C> { engine: &'e mut Engine<C>, }

struct Builder { }

impl<C> Engine<C> {
    pub fn builder() -> Builder {
        Builder {}
    }

    fn get_next_block() -> Block {
        // this is called by BlockBuilder::execute
        todo!()
    }
}

impl Builder {
    // TODO: add other convenience methods for validator config?

    /// Creates a single validator with a randomly generated key.
    fn single_validator(mut self) -> Self {
        todo!();
    }

    /// Explicitly set a list of validators.
    fn with_validators(mut self, _: Vec<Validator>) -> Self {
        todo!();
    }

    /// Returns the currently configured list of validators.
    fn validators(&self) -> &[Validator] {
        todo!();
    }

    // TODO: maybe there's a nicer signature here, figure out later.
    fn with_app_state_bytes(mut self, bytes: Vec<u8>) -> Self { todo!() }

    // this should take the `consensus` thing from pd/main.rs
    async fn init_chain<C>(self, app: C) -> Engine<C> {
        // https://rustdoc.penumbra.zone/main/tower_abci/v037/struct.ServerBuilder.html
        // Engine should be parameterized by the C here
        // init_chain should be parameterized by the C here
        //
        // C: Service<ConsensusRequest, Response = ConsensusResponse, Error = BoxError> + Send + Clone + 'static,
        // C::Future: Send + 'static,
        todo!()
    }

    // fn f(mut self, _: ()) -> Self { todo!() }
}

impl<'e, C> BlockBuilder<'e, C> {
    // TODO: add parameters later
    // TODO: add ways for particular validators to sign/not sign a block

    fn timestamp(mut self) -> Self { todo!() }
    fn relative_timestamp(mut self) -> Self { todo!() }

    // === blah ===

    fn add_tx(mut self, _: Vec<u8>) -> Self { todo!() }

    // === blah ===

    async fn execute(self) -> Self {
        // this should be a higher-level interface, eventually other interfaces should exist
        // to allow us to examine e.g. responses.
        //
        // if we were using the *original* ABCI API this method would:
        // - send a BeginBlock
        // - send a sequence of DeliverTx
        // - send a EndBlock
        // - send a Commit
        // - 
        //
        // in order to form the BeginBlock message, there needs to be a new tendermint block with
        // hash header, commit info, etc. this will create that block before sending those messages
        //
        // 0. create a block
        // 1. ProcessProposal
        // 2. BeginBlock
        // 3. [ DeliverTx ]
        // 4. EndBlock
        // 4. Commit
        todo!()
    }
}
