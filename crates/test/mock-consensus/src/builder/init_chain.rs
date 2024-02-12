use {
    super::*,
    std::time,
    tendermint::{
        block,
        consensus::{
            self,
            params::{AbciParams, ValidatorParams, VersionParams},
        },
        evidence,
        v0_37::abci::{ConsensusRequest, ConsensusResponse},
    },
    tower::{BoxError, Service, ServiceExt},
};

impl Builder {
    /// Consumes this builder, using the provided consensus service.
    pub async fn init_chain<C>(self, mut consensus: C) -> TestNode<C>
    where
        C: Service<ConsensusRequest, Response = ConsensusResponse, Error = BoxError>
            + Send
            + Clone
            + 'static,
        C::Future: Send + 'static,
    {
        let consensus_params = consensus::Params {
            block: block::Size {
                max_bytes: 1,
                max_gas: 1,
                time_iota_ms: 1,
            },
            evidence: evidence::Params {
                max_age_num_blocks: 1,
                max_age_duration: evidence::Duration(time::Duration::from_secs(1)),
                max_bytes: 1,
            },
            validator: ValidatorParams {
                pub_key_types: vec![],
            },
            version: Some(VersionParams { app: 1 }),
            abci: AbciParams {
                vote_extensions_enable_height: None,
            },
        };

        let app_state_bytes = bytes::Bytes::new();
        let init_chain = tendermint::v0_37::abci::request::InitChain {
            time: tendermint::Time::now(),
            chain_id: "test".to_string(), // XXX const here?
            consensus_params,
            validators: vec![],
            app_state_bytes,
            initial_height: 1_u32.into(),
        };
        let ConsensusResponse::InitChain(response) = consensus
            .ready()
            .await
            .unwrap()
            .call(ConsensusRequest::InitChain(init_chain))
            .await
            .unwrap()
        else {
            panic!("unexpected init chain response")
        };

        TestNode {
            consensus,
            _last_app_hash: response.app_hash.as_bytes().to_owned(),
        }
    }
}
