use {
    super::*,
    anyhow::{anyhow, bail},
    bytes::Bytes,
    prost::Message,
    sha2::Digest as _,
    std::{collections::BTreeMap, time},
    tap::TapFallible,
    tendermint::{
        abci::request::InitChain,
        block::{self, Height},
        consensus::{
            self,
            params::{AbciParams, ValidatorParams, VersionParams},
        },
        evidence,
        v0_37::abci::{ConsensusRequest, ConsensusResponse},
        validator::Update,
        vote::Power,
    },
    tendermint_proto::v0_37::types::HashedParams,
    tower::{BoxError, Service, ServiceExt},
    tracing::{debug, error},
};

impl Builder {
    /// Consumes this builder, using the provided consensus service.
    ///
    /// This function returns an error if the builder was not fully initialized, or if the
    /// application could not successfully perform the chain initialization.
    ///
    /// See [`TestNode`] for more information on the consensus service.
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
            keyring,
            on_block,
            initial_timestamp,
            ts_callback,
            chain_id,
            keys: _,
            hardcoded_genesis,
        } = self
        else {
            bail!("builder was not fully initialized")
        };

        let chain_id = tendermint::chain::Id::try_from(
            chain_id.unwrap_or(TestNode::<()>::CHAIN_ID.to_string()),
        )?;

        let timestamp = initial_timestamp.unwrap_or(Time::now());
        let request = match hardcoded_genesis {
            // If there is a hardcoded genesis, ignore whatever else was configured on the builder.
            Some(genesis) => ConsensusRequest::InitChain(InitChain {
                time: genesis.genesis_time,
                chain_id: genesis.chain_id.into(),
                consensus_params: genesis.consensus_params,
                validators: genesis
                    .validators
                    .iter()
                    .map(|v| Update {
                        pub_key: v.pub_key.clone(),
                        power: v.power,
                    })
                    .collect::<Vec<_>>(),
                app_state_bytes: serde_json::to_vec(&genesis.app_state).unwrap().into(),
                initial_height: Height::try_from(genesis.initial_height)?,
            }),
            // Use whatever state was configured on the builder.
            None => {
                Self::init_chain_request(app_state, &keyring, chain_id.clone(), timestamp.clone())?
            }
        };
        let service = consensus
            .ready()
            .await
            .tap_err(|error| error!(?error, "failed waiting for consensus service"))
            .map_err(|_| anyhow!("failed waiting for consensus service"))?;

        let response::InitChain {
            app_hash,
            consensus_params,
            validators,
            ..
        } = match service
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

        // handle validators
        {
            // TODO: implement
            tracing::debug!(?validators, "init_chain ignoring validator updates");
        }

        // The validators aren't properly handled by the mock tendermint
        // and it just maintains this value for the life of the chain right now
        let pk = keyring.iter().next().expect("validator key in keyring").0;
        let pub_key =
            tendermint::PublicKey::from_raw_ed25519(pk.as_bytes()).expect("pub key present");
        let proposer_address = tendermint::validator::Info {
            address: tendermint::account::Id::new(
                <sha2::Sha256 as sha2::Digest>::digest(pk).as_slice()[0..20]
                    .try_into()
                    .expect(""),
            )
            .try_into()?,
            pub_key,
            power: 1i64.try_into()?,
            name: Some("test validator".to_string()),
            proposer_priority: 1i64.try_into()?,
        };

        let hashed_params = HashedParams {
            block_max_bytes: consensus_params
                .as_ref()
                .unwrap()
                .block
                .max_bytes
                .try_into()?,
            block_max_gas: consensus_params.unwrap().block.max_gas,
        };

        Ok(TestNode {
            consensus,
            height: block::Height::from(0_u8),
            last_app_hash: app_hash.as_bytes().to_owned(),
            // TODO: hook this up correctly
            last_validator_set_hash: Some(
                tendermint::validator::Set::new(
                    vec![tendermint::validator::Info {
                        address: proposer_address.address,
                        pub_key,
                        power: Power::try_from(25_000 * 10i64.pow(6))?,
                        name: Some("test validator".to_string()),
                        proposer_priority: 1i64.try_into()?,
                    }],
                    // Same validator as proposer?
                    Some(tendermint::validator::Info {
                        address: proposer_address.address,
                        pub_key,
                        power: Power::try_from(25_000 * 10i64.pow(6))?,
                        name: Some("test validator".to_string()),
                        proposer_priority: 1i64.try_into()?,
                    }),
                )
                .hash(),
            ),
            keyring,
            on_block,
            timestamp,
            ts_callback: ts_callback.unwrap_or(Box::new(default_ts_callback)),
            chain_id,
            consensus_params_hash: sha2::Sha256::digest(hashed_params.encode_to_vec()).to_vec(),
            // No last commit for the genesis block.
            last_commit: None,
        })
    }

    fn init_chain_request(
        app_state_bytes: Bytes,
        keyring: &BTreeMap<ed25519_consensus::VerificationKey, ed25519_consensus::SigningKey>,
        chain_id: tendermint::chain::Id,
        timestamp: Time,
    ) -> Result<ConsensusRequest, anyhow::Error> {
        let consensus_params = Self::consensus_params();

        // TODO: add this to an impl on a keyring
        let pub_keys = keyring
            .iter()
            .map(|(pk, _)| pk)
            .map(|pk| {
                tendermint::PublicKey::from_raw_ed25519(pk.as_bytes()).expect("pub key present")
            })
            .collect::<Vec<_>>();

        Ok(ConsensusRequest::InitChain(InitChain {
            time: timestamp,
            chain_id: chain_id.into(),
            consensus_params,
            validators: pub_keys
                .into_iter()
                .map(|pub_key| tendermint::validator::Update {
                    pub_key,
                    power: 1u64.try_into().unwrap(),
                })
                .collect::<Vec<_>>(),
            app_state_bytes,
            initial_height: 0_u32.into(),
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
