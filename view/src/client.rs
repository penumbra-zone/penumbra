use anyhow::Result;
use futures::TryStreamExt;
use penumbra_crypto::keys::FullViewingKeyHash;
use penumbra_proto::view as pb;
use penumbra_proto::view::view_protocol_client::ViewProtocolClient;
use penumbra_transaction::WitnessData;
use tonic::async_trait;

use crate::NoteRecord;

/// The view protocol is used by a view client, who wants to do some
/// transaction-related actions, to request data from a view service, which is
/// responsible for synchronizing and scanning the public chain state with one
/// or more full viewing keys.
///
/// This trait is a wrapper around the proto-generated [`ViewProtocolClient`]
/// that serves two goals:
///
/// 1. It can use domain types rather than proto-generated types, avoiding conversions;
/// 2. It's easier to write as a trait bound than the `CustodyProtocolClient`,
///   which requires complex bounds on its inner type to
///   enforce that it is a tower `Service`.
#[async_trait(?Send)]
pub trait ViewClient: Sized {
    /// Get the current status of chain sync.
    async fn status(&mut self, fvk_hash: FullViewingKeyHash) -> Result<pb::StatusResponse>;
    /// Queries for notes.
    async fn notes(&mut self, request: pb::NotesRequest) -> Result<Vec<NoteRecord>>;

    /// Returns authentication paths for the given note commitments.
    ///
    /// This method takes a batch of input commitments, rather than just one, so
    /// that the client can get a consistent set of authentication paths to a
    /// common root.  (Otherwise, if a client made multiple requests, the wallet
    /// service could have advanced the note commitment tree state between queries).
    async fn witness(&mut self, request: pb::WitnessRequest) -> Result<WitnessData>;
}

// We need to tell `async_trait` not to add a `Send` bound to the boxed
// futures it generates, because the underlying `CustodyProtocolClient` isn't `Sync`,
// but its `authorize` method takes `&mut self`. This would normally cause a huge
// amount of problems, because non-`Send` futures don't compose well, but as long
// as we're calling the method within an async block on a local mutable variable,
// it should be fine.
#[async_trait(?Send)]
impl<T> ViewClient for ViewProtocolClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: tonic::codegen::Body + Send + 'static,
    T::Error: Into<tonic::codegen::StdError>,
    <T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
{
    async fn status(&mut self, _fvk_hash: FullViewingKeyHash) -> Result<pb::StatusResponse> {
        todo!();
    }

    async fn notes(&mut self, request: pb::NotesRequest) -> Result<Vec<NoteRecord>> {
        let pb_notes: Vec<_> = self
            .notes(tonic::Request::new(request.into()))
            .await?
            .into_inner()
            .try_collect()
            .await?;

        pb_notes.into_iter().map(TryInto::try_into).collect()
    }

    async fn witness(&mut self, _request: pb::WitnessRequest) -> Result<WitnessData> {
        todo!();
    }
}
