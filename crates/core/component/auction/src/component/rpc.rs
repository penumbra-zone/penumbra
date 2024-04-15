#![allow(unused)] // TODO: remove this when filling in the RPCs

use penumbra_proto::{
    core::component::auction::v1alpha1::query_service_server::QueryService, DomainType,
};

use async_stream::try_stream;
use cnidarium::Storage;
use futures::{StreamExt, TryStreamExt};
use std::pin::Pin;
use tonic::Status;
use tracing::instrument;

pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl QueryService for Server {}
