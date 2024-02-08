pub struct TestNode<A> {
    _app: A,
    _last_app_hash: Vec<u8>,
}

pub mod block {
    use crate::TestNode;

    struct _Builder<'e, C> {
        engine: &'e mut TestNode<C>,
    }
}

pub struct Builder;

impl<A> TestNode<A> {
    pub fn builder() -> Builder {
        Builder
    }
}

impl Builder {
    // TODO: add other convenience methods for validator config?

    /// Creates a single validator with a randomly generated key.
    pub fn single_validator(self) -> Self {
        todo!();
    }

    pub fn app_state(self, _: ()) -> Self {
        todo!()
    }

    pub fn app_state_bytes(self, _: Vec<u8>) -> Self {
        todo!()
    }

    // this should take the `consensus` thing from pd/main.rs
    pub async fn init_chain<A>(self, _: A) -> TestNode<A> {
        // https://rustdoc.penumbra.zone/main/tower_abci/v037/struct.ServerBuilder.html
        // Engine should be parameterized by the C here
        // init_chain should be parameterized by the C here
        //
        // C: Service<ConsensusRequest, Response = ConsensusResponse, Error = BoxError> + Send + Clone + 'static,
        // C::Future: Send + 'static,
        todo!()
    }
}
