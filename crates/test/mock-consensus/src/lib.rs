pub struct TestNode<S> {
    #[allow(dead_code)]
    abci_server: S,
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

    pub async fn init_chain<S>(self, abci_server: S) -> TestNode<S> {
        TestNode {
            abci_server,
            _last_app_hash: vec![],
        }
    }
}
