//! Facilities related to the Penumbra app's ABCI server.

use {
    self::{
        consensus::Consensus, events::EventIndexLayer, info::Info, mempool::Mempool,
        snapshot::Snapshot,
    },
    cnidarium::Storage,
    penumbra_sdk_tower_trace::trace::request_span,
    tendermint::v0_37::abci::{
        ConsensusRequest, ConsensusResponse, MempoolRequest, MempoolResponse,
    },
    tower_abci::{v037::Server, BoxError},
};

pub mod consensus;
pub mod info;
pub mod mempool;
pub mod snapshot;

mod events;

/// Returns a newly instantiated ABCI [`Server`], backed by the provided [`Storage`].
pub fn new(
    storage: Storage,
) -> Server<
    // These bounds ensure that the server can be bound to a TCP port, or a Unix socket.
    impl tower_service::Service<
            ConsensusRequest,
            Response = ConsensusResponse,
            Error = BoxError,
            Future = impl Send + 'static,
        > + Send
        + Clone
        + 'static,
    impl tower_service::Service<
            MempoolRequest,
            Response = MempoolResponse,
            Error = BoxError,
            Future = impl Send + 'static,
        > + Send
        + Clone
        + 'static,
    Info,
    Snapshot,
> {
    let consensus = tower::ServiceBuilder::new()
        .layer(request_span::layer(|req: &ConsensusRequest| {
            use penumbra_sdk_tower_trace::v037::RequestExt;
            req.create_span()
        }))
        .layer(EventIndexLayer::index_all())
        .service(Consensus::new(storage.clone()));
    let mempool = tower::ServiceBuilder::new()
        .layer(request_span::layer(|req: &MempoolRequest| {
            use penumbra_sdk_tower_trace::v037::RequestExt;
            req.create_span()
        }))
        .service(tower_actor::Actor::new(10, |queue: _| {
            Mempool::new(storage.clone(), queue).run()
        }));
    let info = Info::new(storage.clone());
    let snapshot = Snapshot {};

    tower_abci::v037::Server::builder()
        .consensus(consensus)
        .snapshot(snapshot)
        .mempool(mempool)
        .info(info.clone())
        .finish()
        // Safety: the consensus, snapshot, mempool, and info services have all been provided
        // to the builder above.
        .expect("all components of abci have been provided")
}

#[cfg(test)]
mod bounds_test {
    /// Show that a server satisfies the trait bounds needed to listen on a TCP port.
    #[allow(dead_code, unreachable_code, unused_variables)]
    async fn servers_can_listen() {
        let storage: cnidarium::Storage = todo!();
        let addr: std::net::SocketAddr = todo!();
        let server = super::new(storage).listen_tcp(addr);
        drop(server);
    }
}
