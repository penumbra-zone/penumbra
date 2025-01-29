use std::{future::Future, pin::Pin, task::Context};

use anyhow::Result;
use futures::FutureExt;
use regex::RegexSet;
use tendermint::abci::Event;
use tendermint::v0_37::abci::{ConsensusRequest as Request, ConsensusResponse as Response};
use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub struct EventIndex<S> {
    svc: S,
    config: EventIndexLayer,
}

#[derive(Debug, Clone, Default)]
pub struct EventIndexLayer {
    /// A set of regexes matching event keys that should be set to `index=true`.
    ///
    /// Takes priority over `no_index` matches.
    pub index: RegexSet,
    /// A set of regexes matching event keys that should be set to `index=false`.
    pub no_index: RegexSet,
}

impl EventIndexLayer {
    /// Convenience constructor to force every event attribute to be indexed.
    pub fn index_all() -> Self {
        Self {
            index: RegexSet::new([""]).expect("empty regex should always parse"),
            no_index: RegexSet::empty(),
        }
    }

    fn adjust_events(&self, events: &mut [Event]) {
        for e in events.iter_mut() {
            for attr in e.attributes.iter_mut() {
                // Perform matching on a nested key in the same format used by
                // the cosmos SDK: https://docs.cosmos.network/main/core/config
                // e.g., "message.sender", "message.recipient"

                match attr.key_str() {
                    Ok(key) => {
                        let nested_key = format!("{}.{}", e.kind, key);

                        if self.no_index.is_match(&nested_key) {
                            attr.set_index(false);
                        }

                        // This comes second so that explicit index requests take priority over no-index requests.
                        if self.index.is_match(&nested_key) {
                            attr.set_index(true);
                        }
                    }
                    _ => {
                        // The key is not valid UTF-8, so we can't match it, we skip it.
                        // This should be unreachable, as the key is always a valid UTF-8 string
                        // for tendermint/cometbft > 0.34
                        tracing::warn!("event attribute key is not valid UTF-8");
                    }
                }
            }
        }
    }
}

impl<S> Layer<S> for EventIndexLayer
where
    S: Service<Request, Response = Response>,
    S::Future: Send + 'static,
{
    type Service = EventIndex<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EventIndex {
            svc: inner,
            config: self.clone(),
        }
    }
}

impl<S> Service<Request> for EventIndex<S>
where
    S: Service<Request, Response = Response>,
    S::Future: Send + 'static,
{
    type Error = S::Error;
    type Response = Response;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let rsp = self.svc.call(req);
        let config = self.config.clone();

        async move {
            let mut rsp = rsp.await?;
            match rsp {
                // No events.
                Response::InitChain(_) => {}
                Response::Commit(_) => {}
                Response::PrepareProposal(_) => {}
                Response::ProcessProposal(_) => {}
                // These responses have events.
                Response::BeginBlock(ref mut msg) => config.adjust_events(&mut msg.events),
                Response::DeliverTx(ref mut msg) => config.adjust_events(&mut msg.events),
                Response::EndBlock(ref mut msg) => config.adjust_events(&mut msg.events),
            }
            Ok(rsp)
        }
        .boxed()
    }
}
