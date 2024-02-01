use async_trait::async_trait;
use cnidarium::StateWrite;

use crate::{state_key, CommitmentSource};

/// A helper trait for placing a `CommitmentSource` as ambient context during execution.
#[async_trait]
pub trait SourceContext: StateWrite {
    fn put_current_source(&mut self, source: Option<CommitmentSource>) {
        if let Some(source) = source {
            self.object_put(state_key::ambient::current_source(), source)
        } else {
            self.object_delete(state_key::ambient::current_source())
        }
    }

    fn get_current_source(&self) -> Option<CommitmentSource> {
        self.object_get(state_key::ambient::current_source())
    }

    /// Sets a mock source, for testing.
    ///
    /// The `counter` field allows distinguishing hashes at different stages of the test.
    fn put_mock_source(&mut self, counter: u8) {
        self.put_current_source(Some(CommitmentSource::Transaction {
            id: Some([counter; 32]),
        }))
    }
}
impl<T: StateWrite + ?Sized> SourceContext for T {}
