use {
    super::*,
    anyhow::{anyhow, bail},
    bytes::Bytes,
    std::time,
    tap::TapFallible,
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
    tracing::{debug, error},
};

impl Builder {
    /// Consumes this builder, using the provided consensus service.
    pub async fn init_chain<C>(self, mut consensus: C) -> Result<TestNode<C>, anyhow::Error>
    where
        C: Service<ConsensusRequest, Response = ConsensusResponse, Error = BoxError>
            + Send
            + Clone
            + 'static,
        C::Future: Send + 'static,
        C::Error: Sized,
    {
        use tendermint::v0_37::abci::response;

        let Self {
            app_state: Some(app_state),
            identity_key: Some(identity_key),
            extensions: _,
        } = self
        else {
            bail!("builder was not fully initialized")
        };

        let request = Self::init_chain_request(app_state)?;
        let service = consensus
            .ready()
            .await
            .tap_err(|error| error!(?error, "failed waiting for consensus service"))
            .map_err(|_| anyhow!("failed waiting for consensus service"))?;

        let response::InitChain { app_hash, .. } = match service
            .call(request)
            .await
            .tap_ok(|resp| debug!(?resp, "received response from consensus service"))
            .tap_err(|error| error!(?error, "consensus service returned error"))
            .map_err(|_| anyhow!("consensus service returned error"))?
        {
            ConsensusResponse::InitChain(resp) => resp,
            response => {
                error!(?response, "unexpected InitChain response");
                bail!("unexpected InitChain response");
            }
        };

        Ok(TestNode {
            consensus,
            height: block::Height::from(0_u8),
            last_app_hash: app_hash.as_bytes().to_owned(),
            identity_key: identity_key.try_into()?,
        })
    }

    fn init_chain_request(app_state_bytes: Bytes) -> Result<ConsensusRequest, anyhow::Error> {
        use tendermint::v0_37::abci::request::InitChain;
        let chain_id = TestNode::<()>::CHAIN_ID.to_string();
        let consensus_params = Self::consensus_params();
        Ok(ConsensusRequest::InitChain(InitChain {
            time: tendermint::Time::now(),
            chain_id,
            consensus_params,
            validators: vec![],
            app_state_bytes,
            initial_height: 1_u32.into(),
        }))
    }

    fn consensus_params() -> consensus::Params {
        consensus::Params {
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
        }
    }
}
