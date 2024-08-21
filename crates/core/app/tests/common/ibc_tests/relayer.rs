use {
    super::TestNodeWithIBC,
    anyhow::{anyhow, Result},
    ibc_proto::ibc::core::{
        client::v1::{QueryClientStateRequest, QueryConsensusStateRequest},
        connection::v1::QueryConnectionRequest,
    },
    ibc_types::lightclients::tendermint::client_state::ClientState as TendermintClientState,
    ibc_types::{
        core::{
            client::{
                msgs::{MsgCreateClient, MsgUpdateClient},
                Height,
            },
            commitment::{MerkleProof, MerkleRoot},
            connection::{
                msgs::{
                    MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
                    MsgConnectionOpenTry,
                },
                ConnectionEnd, Counterparty, State as ConnectionState, Version,
            },
        },
        lightclients::tendermint::{
            client_state::AllowUpdate, consensus_state::ConsensusState,
            header::Header as TendermintHeader, TrustThreshold,
        },
        DomainType as _,
    },
    penumbra_ibc::{
        component::ConnectionStateReadExt as _, IbcRelay, IBC_COMMITMENT_PREFIX, IBC_PROOF_SPECS,
    },
    penumbra_proto::{util::tendermint_proxy::v1::GetBlockByHeightRequest, DomainType},
    penumbra_stake::state_key::chain,
    penumbra_transaction::{TransactionParameters, TransactionPlan},
    prost::Message as _,
    sha2::Digest,
    std::time::Duration,
    tendermint::Time,
};
#[allow(unused)]
pub struct MockRelayer {
    pub chain_a_ibc: TestNodeWithIBC,
    pub chain_b_ibc: TestNodeWithIBC,
}

#[allow(unused)]
impl MockRelayer {
    pub async fn get_connection_states(&mut self) -> Result<(ConnectionState, ConnectionState)> {
        let connection_on_a_response = self
            .chain_a_ibc
            .ibc_connection_query_client
            .connection(QueryConnectionRequest {
                connection_id: self.chain_a_ibc.connection_id.to_string(),
            })
            .await?
            .into_inner();
        let connection_on_b_response = self
            .chain_b_ibc
            .ibc_connection_query_client
            .connection(QueryConnectionRequest {
                connection_id: self.chain_b_ibc.connection_id.to_string(),
            })
            .await?
            .into_inner();

        Ok(
            match (
                connection_on_a_response.connection,
                connection_on_b_response.connection,
            ) {
                (Some(connection_a), Some(connection_b)) => {
                    let connection_a: ConnectionEnd = connection_a.try_into().unwrap();
                    let connection_b: ConnectionEnd = connection_b.try_into().unwrap();
                    (connection_a.state, connection_b.state)
                }
                (None, None) => (
                    ConnectionState::Uninitialized,
                    ConnectionState::Uninitialized,
                ),
                (None, Some(connection_b)) => {
                    let connection_b: ConnectionEnd = connection_b.try_into().unwrap();
                    (ConnectionState::Uninitialized, connection_b.state)
                }
                (Some(connection_a), None) => {
                    let connection_a: ConnectionEnd = connection_a.try_into().unwrap();
                    (connection_a.state, ConnectionState::Uninitialized)
                }
            },
        )
    }

    pub async fn _handshake(&mut self) -> Result<(), anyhow::Error> {
        // The IBC connection handshake has four steps (Init, Try, Ack, Confirm).
        // https://github.com/penumbra-zone/hermes/blob/a34a11fec76de3b573b539c237927e79cb74ec00/crates/relayer/src/connection.rs#L672
        // https://github.com/cosmos/ibc/blob/main/spec/core/ics-003-connection-semantics/README.md#opening-handshake

        self._sync_chains().await?;

        let (a_state, b_state) = self.get_connection_states().await?;
        assert!(
            a_state == ConnectionState::Uninitialized && b_state == ConnectionState::Uninitialized
        );

        // 1: send the Init message to chain A
        {
            tracing::info!("Send Init to chain A");
            self._build_and_send_connection_open_init().await?;
        }

        let (a_state, b_state) = self.get_connection_states().await?;
        assert!(a_state == ConnectionState::Init && b_state == ConnectionState::Uninitialized);

        self._sync_chains().await?;

        // 2. send the OpenTry message to chain B
        {
            tracing::info!("send OpenTry to chain B");
            self._build_and_send_connection_open_try().await?;
        }

        let (a_state, b_state) = self.get_connection_states().await?;
        assert!(a_state == ConnectionState::Init && b_state == ConnectionState::TryOpen);

        self._sync_chains().await?;

        // 3. Send the OpenAck message to chain A
        {
            tracing::info!("send OpenAck to chain A");
            self._build_and_send_connection_open_ack().await?;
        }

        let (a_state, b_state) = self.get_connection_states().await?;
        assert!(a_state == ConnectionState::Open && b_state == ConnectionState::TryOpen);

        self._sync_chains().await?;

        // 4. Send the OpenConfirm message to chain B
        {
            tracing::info!("send OpenConfirm to chain B");
            self._build_and_send_connection_open_confirm().await?;
        }

        let (a_state, b_state) = self.get_connection_states().await?;
        assert!(a_state == ConnectionState::Open && b_state == ConnectionState::Open);

        // Ensure the chain timestamps remain in sync
        self._sync_chains().await?;

        Ok(())
    }

    pub async fn _create_clients(&mut self) -> Result<(), anyhow::Error> {
        self._sync_chains().await?;
        // helper function to create client for chain B on chain A
        async fn _create_client_inner(
            chain_a_ibc: &mut TestNodeWithIBC,
            chain_b_ibc: &mut TestNodeWithIBC,
        ) -> Result<()> {
            let pk = chain_b_ibc
                .node
                .keyring()
                .iter()
                .next()
                .expect("validator key in keyring")
                .0;
            let proposer_address = tendermint::account::Id::new(
                <sha2::Sha256 as sha2::Digest>::digest(pk).as_slice()[0..20]
                    .try_into()
                    .expect(""),
            );
            let pub_key =
                tendermint::PublicKey::from_raw_ed25519(pk.as_bytes()).expect("pub key present");
            // RIGHT HERE everything is correct.
            // Creating client on penumbra-test-chain-b with height 2 and merkle root 58055...
            // later on, though: proof root for height 3: 58055...
            // that is not right
            println!(
                "Creating client on {} with height {} and merkle root {}",
                chain_a_ibc.chain_id,
                chain_b_ibc.get_latest_height().await?.revision_height,
                hex::encode(chain_b_ibc.node.last_app_hash())
            );
            // Create the client for chain B on chain A.
            let plan = {
                let ibc_msg = IbcRelay::CreateClient(MsgCreateClient {
                    // Chain B will be signing messages to chain A
                    signer: chain_b_ibc.signer.clone(),
                    client_state: ibc_types::lightclients::tendermint::client_state::ClientState {
                        // Chain ID of the client state is for the counterparty
                        chain_id: chain_b_ibc.chain_id.clone().into(),
                        trust_level: TrustThreshold {
                            numerator: 1,
                            denominator: 3,
                        },
                        trusting_period: Duration::from_secs(120_000),
                        unbonding_period: Duration::from_secs(240_000),
                        max_clock_drift: Duration::from_secs(5),
                        // The latest_height is for chain B
                        latest_height: chain_b_ibc.get_latest_height().await?,
                        // The ICS02 validation is hardcoded to expect 2 proof specs
                        // (root and substore, see [`penumbra_ibc::component::ics02_validation`]).
                        proof_specs: IBC_PROOF_SPECS.to_vec(),
                        upgrade_path: vec!["upgrade".to_string(), "upgradedIBCState".to_string()],
                        allow_update: AllowUpdate {
                            after_expiry: false,
                            after_misbehaviour: false,
                        },
                        frozen_height: None,
                    }
                    .into(),
                    consensus_state:
                        ibc_types::lightclients::tendermint::consensus_state::ConsensusState {
                            timestamp: *chain_b_ibc.node.timestamp(),
                            root: MerkleRoot {
                                hash: chain_b_ibc.node.last_app_hash().to_vec(),
                            },
                            next_validators_hash: (*chain_b_ibc
                                .node
                                .last_validator_set_hash()
                                .unwrap())
                            .into(),
                        }
                        .into(),
                })
                .into();
                TransactionPlan {
                    actions: vec![ibc_msg],
                    // Now fill out the remaining parts of the transaction needed for verification:
                    memo: None,
                    detection_data: None, // We'll set this automatically below
                    transaction_parameters: TransactionParameters {
                        chain_id: chain_a_ibc.chain_id.clone(),
                        ..Default::default()
                    },
                }
            };
            let tx = chain_a_ibc
                .client()
                .await?
                .witness_auth_build(&plan)
                .await?;

            // Create the client for chain B on chain A.
            chain_a_ibc
                .node
                .block()
                .with_data(vec![tx.encode_to_vec()])
                .execute()
                .await?;

            Ok(())
        }

        // Each chain will need a client created corresponding to its IBC connection with the other chain:
        _create_client_inner(&mut self.chain_a_ibc, &mut self.chain_b_ibc).await?;
        _create_client_inner(&mut self.chain_b_ibc, &mut self.chain_a_ibc).await?;

        Ok(())
    }

    // helper function to build ConnectionOpenInit to chain A
    pub async fn _build_and_send_connection_open_init(&mut self) -> Result<()> {
        self._sync_chains().await?;
        let chain_a_ibc = &mut self.chain_a_ibc;
        let chain_b_ibc = &mut self.chain_b_ibc;
        let plan = {
            let ibc_msg = IbcRelay::ConnectionOpenInit(MsgConnectionOpenInit {
                client_id_on_a: chain_a_ibc.client_id.clone(),
                counterparty: chain_a_ibc.counterparty.clone(),
                version: Some(chain_a_ibc.version.clone()),
                delay_period: Duration::from_secs(1),
                signer: chain_b_ibc.signer.clone(),
            })
            .into();
            TransactionPlan {
                actions: vec![ibc_msg],
                // Now fill out the remaining parts of the transaction needed for verification:
                memo: None,
                detection_data: None, // We'll set this automatically below
                transaction_parameters: TransactionParameters {
                    chain_id: chain_a_ibc.chain_id.clone(),
                    ..Default::default()
                },
            }
        };
        let tx = chain_a_ibc
            .client()
            .await?
            .witness_auth_build(&plan)
            .await?;

        // Execute the transaction, applying it to the chain state.
        let pre_tx_snapshot = chain_a_ibc.storage.latest_snapshot();
        chain_a_ibc
            .node
            .block()
            .with_data(vec![tx.encode_to_vec()])
            .execute()
            .await?;
        let post_tx_snapshot = chain_a_ibc.storage.latest_snapshot();

        // validate the connection state is now "init"
        {
            // Connection should not exist pre-commit
            assert!(pre_tx_snapshot
                .get_connection(&chain_a_ibc.connection_id)
                .await?
                .is_none(),);

            // Post-commit, the connection should be in the "init" state.
            let connection = post_tx_snapshot
                .get_connection(&chain_a_ibc.connection_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no connection with the specified ID {} exists",
                        &chain_a_ibc.connection_id
                    )
                })?;

            assert_eq!(connection.state.clone(), ConnectionState::Init);

            chain_a_ibc.connection = Some(connection.clone());
        }

        Ok(())
    }

    pub async fn handshake(&mut self) -> Result<(), anyhow::Error> {
        // Open a connection on each chain to the other chain.
        // This is accomplished by following the ICS-003 spec for connection handshakes.

        // The Clients need to be created on each chain prior to the handshake.
        self._create_clients().await?;
        // The handshake is a multi-step process, this call will ratchet through the steps.
        self._handshake().await?;

        Ok(())
    }

    // helper function to sync the chain times
    pub async fn _sync_chains(&mut self) -> Result<()> {
        let mut chain_a_time = self.chain_a_ibc.node.timestamp();
        let mut chain_b_time = self.chain_b_ibc.node.timestamp();

        while chain_a_time < chain_b_time {
            self.chain_a_ibc.node.block().execute().await?;
            chain_a_time = self.chain_a_ibc.node.timestamp();
        }
        while chain_b_time < chain_a_time {
            self.chain_b_ibc.node.block().execute().await?;
            chain_b_time = self.chain_b_ibc.node.timestamp();
        }

        chain_a_time = self.chain_a_ibc.node.timestamp();
        chain_b_time = self.chain_b_ibc.node.timestamp();
        assert_eq!(chain_a_time, chain_b_time);

        Ok(())
    }

    // tell chain b about chain a
    pub async fn _build_and_send_update_client_b(&mut self, target_height: Height) -> Result<()> {
        tracing::info!(
            "send update client for chain {} to chain {} about height {}",
            self.chain_a_ibc.chain_id,
            self.chain_b_ibc.chain_id,
            target_height
        );
        // reverse these because we're sending to chain B
        let chain_a_ibc = &mut self.chain_b_ibc;
        let chain_b_ibc = &mut self.chain_a_ibc;

        _build_and_send_update_client(chain_a_ibc, chain_b_ibc, target_height).await
    }

    // helper function to build UpdateClient to send to chain A
    pub async fn _build_and_send_update_client_a(&mut self, target_height: Height) -> Result<()> {
        tracing::info!(
            "send update client for chain {} to chain {} about height {}",
            self.chain_b_ibc.chain_id,
            self.chain_a_ibc.chain_id,
            target_height
        );
        let chain_a_ibc = &mut self.chain_a_ibc;
        let chain_b_ibc = &mut self.chain_b_ibc;

        _build_and_send_update_client(chain_a_ibc, chain_b_ibc, target_height).await
    }

    // Send an ACK message to chain A
    // https://github.com/penumbra-zone/hermes/blob/a34a11fec76de3b573b539c237927e79cb74ec00/crates/relayer/src/connection.rs#L1126
    async fn _build_and_send_connection_open_ack(&mut self) -> Result<()> {
        let chain_b_connection_id = self.chain_b_ibc.connection_id.clone();
        let chain_a_connection_id = self.chain_a_ibc.connection_id.clone();
        let connection_of_a_on_b_response = self
            .chain_b_ibc
            .ibc_connection_query_client
            .connection(QueryConnectionRequest {
                connection_id: chain_a_connection_id.to_string(),
            })
            .await?
            .into_inner();

        // Build message(s) for updating client on source
        let src_client_target_height = self.chain_a_ibc.get_latest_height().await?;
        self._build_and_send_update_client_a(src_client_target_height)
            .await?;
        // Build message(s) for updating client on destination
        let dst_client_target_height = self.chain_b_ibc.get_latest_height().await?;
        self._build_and_send_update_client_b(dst_client_target_height)
            .await?;

        let client_state_of_a_on_b_response = self
            .chain_b_ibc
            .ibc_client_query_client
            .client_state(QueryClientStateRequest {
                client_id: self.chain_a_ibc.client_id.to_string(),
            })
            .await?
            .into_inner();
        let consensus_state_of_a_on_b_response = self
            .chain_b_ibc
            .ibc_client_query_client
            .consensus_state(QueryConsensusStateRequest {
                client_id: self.chain_a_ibc.client_id.to_string(),
                revision_number: 0,
                revision_height: 0,
                latest_height: true,
            })
            .await?
            .into_inner();

        let plan = {
            // This mocks the relayer constructing a connection open try message on behalf
            // of the counterparty chain.
            // we can't directly construct this because one of the struct fields is private
            // and it's not from this crate, but we _can_ create the proto type and then convert it!
            let proto_ack = ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck {
                connection_id: self.chain_a_ibc.connection_id.to_string(),
                counterparty_connection_id: chain_b_connection_id.to_string(),
                version: Some(Version::default().into()),
                client_state: Some(
                    client_state_of_a_on_b_response
                        .clone()
                        .client_state
                        .unwrap(),
                ),
                proof_height: Some(connection_of_a_on_b_response.clone().proof_height.unwrap()),
                proof_try: connection_of_a_on_b_response.proof,
                proof_client: client_state_of_a_on_b_response.clone().proof,
                proof_consensus: consensus_state_of_a_on_b_response.proof,
                // consensus height of a on b (the height chain b's ibc client trusts chain a at)
                consensus_height: Some(
                    ibc_types::lightclients::tendermint::client_state::ClientState::try_from(
                        client_state_of_a_on_b_response
                            .clone()
                            .client_state
                            .unwrap(),
                    )?
                    .latest_height
                    .into(),
                ),
                signer: self.chain_b_ibc.signer.clone(),
                // optional field, don't include
                host_consensus_state_proof: vec![],
            };
            let ibc_msg =
                IbcRelay::ConnectionOpenAck(MsgConnectionOpenAck::try_from(proto_ack)?).into();
            TransactionPlan {
                actions: vec![ibc_msg],
                // Now fill out the remaining parts of the transaction needed for verification:
                memo: None,
                detection_data: None, // We'll set this automatically below
                transaction_parameters: TransactionParameters {
                    chain_id: self.chain_a_ibc.chain_id.clone(),
                    ..Default::default()
                },
            }
        };
        let tx = self
            .chain_a_ibc
            .client()
            .await?
            .witness_auth_build(&plan)
            .await?;

        // Execute the transaction, applying it to the chain state.
        let pre_tx_snapshot = self.chain_a_ibc.storage.latest_snapshot();
        self.chain_a_ibc
            .node
            .block()
            .with_data(vec![tx.encode_to_vec()])
            .execute()
            .await?;
        let post_tx_snapshot = self.chain_a_ibc.storage.latest_snapshot();

        // validate the connection state is now "OPEN"
        {
            // Connection should be in INIT pre-commit
            let connection = pre_tx_snapshot
                .get_connection(&self.chain_a_ibc.connection_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no connection with the specified ID {} exists",
                        &self.chain_a_ibc.connection_id
                    )
                })?;

            assert_eq!(connection.state, ConnectionState::Init);

            // Post-commit, the connection should be in the "OPEN" state.
            let connection = post_tx_snapshot
                .get_connection(&self.chain_a_ibc.connection_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no connection with the specified ID {} exists",
                        &self.chain_a_ibc.connection_id
                    )
                })?;

            assert_eq!(connection.state, ConnectionState::Open);

            self.chain_a_ibc.connection = Some(connection);
        }

        Ok(())
    }

    // helper function to build ConnectionOpenTry to send to chain B
    // at this point chain A is in INIT state and chain B has no state
    // after this, chain A will be in INIT and chain B will be in TRYOPEN state.
    pub async fn _build_and_send_connection_open_try(&mut self) -> Result<()> {
        // This is a load-bearing block execution that should be removed
        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        self._sync_chains().await?;

        let src_connection = self
            .chain_a_ibc
            .ibc_connection_query_client
            .connection(QueryConnectionRequest {
                connection_id: self.chain_a_ibc.connection_id.to_string(),
            })
            .await?
            .into_inner();

        let src_client_target_height = self.chain_b_ibc.get_latest_height().await?;
        let client_msgs = self
            ._build_and_send_update_client_a(src_client_target_height)
            .await?;

        // Make sure chain B has a client state for this height
        println!("UPDATE1");
        // self._build_and_send_update_client_b().await?;
        // self._sync_chains().await?;

        // the height chain b's client for chain a should have state for
        let chain_b_client_a_target_height = self.chain_a_ibc.get_latest_height().await?;

        let client_state_of_b_on_a_response = self
            .chain_a_ibc
            .ibc_client_query_client
            .client_state(QueryClientStateRequest {
                client_id: self.chain_b_ibc.client_id.to_string(),
            })
            .await?
            .into_inner();
        let connection_of_b_on_a_response = self
            .chain_a_ibc
            .ibc_connection_query_client
            .connection(QueryConnectionRequest {
                connection_id: self.chain_b_ibc.connection_id.to_string(),
            })
            .await?
            .into_inner();

        let proofs_height_on_a: Height = connection_of_b_on_a_response
            .proof_height
            .clone()
            .unwrap()
            .try_into()?;
        // the proof was included in proof_height, but the root for the proof
        // will be available in the next block's header, so we need to increment
        // TODO: what do
        // let proofs_height_on_a = proofs_height_on_a.increment();
        println!(
            "fetching consensus state at height: {:?}",
            proofs_height_on_a
        );

        assert_eq!(
            client_state_of_b_on_a_response.proof_height,
            connection_of_b_on_a_response.proof_height
        );

        let proof_client_state_of_b_on_a =
            MerkleProof::decode(client_state_of_b_on_a_response.clone().proof.as_slice())?;
        let proof_conn_end_on_a =
            MerkleProof::decode(connection_of_b_on_a_response.clone().proof.as_slice())?;

        let existence_proofs: Vec<(Vec<u8>, Vec<u8>)> = proof_conn_end_on_a
            .proofs
            .iter()
            .map(|proof| {
                let p = proof.proof.clone().unwrap();
                match p {
                    ics23::commitment_proof::Proof::Exist(p) => (p.key, p.value),
                    _ => (vec![], vec![]),
                }
            })
            .inspect(|(k, v)| {
                println!(
                    "proof_conn_end_on_a: k {} v {}",
                    hex::encode(k),
                    hex::encode(v)
                );
            })
            .collect();

        // https://github.com/penumbra-zone/hermes/blob/a34a11fec76de3b573b539c237927e79cb74ec00/crates/relayer/src/connection.rs#L1010
        // https://github.com/penumbra-zone/hermes/blob/main/crates/relayer/src/foreign_client.rs#L1144
        // Send an update to both sides to ensure they are up to date
        // Build message(s) for updating client on source
        // chain B needs to know about chain A at the proof height
        self._build_and_send_update_client_b(proofs_height_on_a.try_into()?)
            .await?;
        println!(
            "client state target height: {:?}",
            chain_b_client_a_target_height
        );

        // TODO: too side-effecty?
        self.chain_b_ibc.counterparty.connection_id = Some(self.chain_a_ibc.connection_id.clone());
        self.chain_a_ibc.counterparty.connection_id = Some(self.chain_b_ibc.connection_id.clone());

        // Build message(s) for updating client on destination
        println!(
            "proof height: {:?}",
            connection_of_b_on_a_response.proof_height
        );

        // d01e8818de18fb050c7aff28a59d430db07802f91c14c4370cf8be785381d4c7 is trusted (for height 6)
        // but the proof comes in for ae0565596d0d8fb9019815400868ecf002b3aa48f4e0134e116f3184cfa49c4a (height 7)
        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        println!(
            "chain b latest height: {:?}",
            self.chain_b_ibc.get_latest_height().await?
        );
        // https://github.com/penumbra-zone/hermes/blob/a34a11fec76de3b573b539c237927e79cb74ec00/crates/relayer/src/connection.rs#L943

        let mut latest_client_state =
            ibc_types::lightclients::tendermint::client_state::ClientState::try_from(
                self.chain_b_ibc
                    .ibc_client_query_client
                    .client_state(QueryClientStateRequest {
                        client_id: self.chain_a_ibc.client_id.to_string(),
                    })
                    .await?
                    .into_inner()
                    .client_state
                    .unwrap(),
            )?;

        let consensus_state_of_b_on_a_response = self
            .chain_a_ibc
            .ibc_client_query_client
            .consensus_state(QueryConsensusStateRequest {
                client_id: self.chain_b_ibc.client_id.to_string(),
                revision_number: src_client_target_height.revision_number,
                // use the same height for the consensus state as the connection
                revision_height: src_client_target_height.revision_height,
                latest_height: false,
            })
            .await?
            .into_inner();
        println!(
            "consensus_state_of_b_on_a_response: {:?}",
            consensus_state_of_b_on_a_response
        );

        // Then construct the ConnectionOpenTry message
        let proof_consensus_state_of_b_on_a =
            MerkleProof::decode(consensus_state_of_b_on_a_response.clone().proof.as_slice())?;

        self._build_and_send_update_client_b(proofs_height_on_a)
            .await?;
        self._sync_chains().await?;

        let cs: TendermintClientState = client_state_of_b_on_a_response
            .clone()
            .client_state
            .unwrap()
            .try_into()?;
        // let cs = ibc_types::lightclients::tendermint::client_state::ClientState::try_from(
        //     client_state_of_b_on_a_response
        //         .clone()
        //         .client_state
        //         .unwrap(),
        // )?;
        println!("cs: {:?}", cs);
        println!("before send cs latest height: {}", cs.latest_height);
        let consensus_state_of_b_on_a_proof_height =
            consensus_state_of_b_on_a_response.proof_height.unwrap();
        let plan = {
            // This mocks the relayer constructing a connection open try message on behalf
            // of the counterparty chain.
            #[allow(deprecated)]
            let ibc_msg = IbcRelay::ConnectionOpenTry(MsgConnectionOpenTry {
                // Counterparty is chain A.
                counterparty: Counterparty {
                    client_id: self.chain_a_ibc.client_id.clone(),
                    connection_id: Some(self.chain_a_ibc.connection_id.clone()),
                    prefix: IBC_COMMITMENT_PREFIX.to_owned(),
                },
                delay_period: Duration::from_secs(1),
                signer: self.chain_a_ibc.signer.clone(),
                client_id_on_b: self.chain_b_ibc.client_id.clone(),
                client_state_of_b_on_a: client_state_of_b_on_a_response
                    .client_state
                    .expect("client state present"),
                versions_on_a: vec![Version::default()],
                proof_conn_end_on_a,
                proof_client_state_of_b_on_a,
                proof_consensus_state_of_b_on_a,
                proofs_height_on_a: Height {
                    revision_height: consensus_state_of_b_on_a_proof_height.revision_height,
                    revision_number: consensus_state_of_b_on_a_proof_height.revision_number,
                },
                consensus_height_of_b_on_a: Height {
                    revision_height: src_client_target_height.revision_height,
                    revision_number: src_client_target_height.revision_number,
                },
                // this seems to be an optional proof
                proof_consensus_state_of_b: None,
                // deprecated
                previous_connection_id: "".to_string(),
            })
            .into();
            TransactionPlan {
                actions: vec![ibc_msg],
                // Now fill out the remaining parts of the transaction needed for verification:
                memo: None,
                detection_data: None, // We'll set this automatically below
                transaction_parameters: TransactionParameters {
                    chain_id: self.chain_b_ibc.chain_id.clone(),
                    ..Default::default()
                },
            }
        };
        let tx = self
            .chain_b_ibc
            .client()
            .await?
            .witness_auth_build(&plan)
            .await?;

        // Execute the transaction, applying it to the chain state.
        let pre_tx_snapshot = self.chain_b_ibc.storage.latest_snapshot();

        // validate the chain b pre-tx storage root hash is what we expect:
        let pre_tx_hash = pre_tx_snapshot.root_hash().await?;
        // assert_eq!(
        //     hex::encode(pre_tx_hash),
        //     "27572242ba7935d5c9acf4ee162fc1418be1f749677aecd0eb11d832bb8d8613".to_string()
        // );

        // Validate the tx hash is what we expect:
        let tx_hash = sha2::Sha256::digest(&tx.encode_to_vec());
        // assert_eq!(
        //     hex::encode(tx_hash),
        //     "8b741a3cfb2bcd4cf665780d3f9e18a6b954f15912591c1acc6d21de015848dc".to_string()
        // );

        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;

        // execute the transaction containing the opentry message
        self.chain_b_ibc
            .node
            .block()
            .with_data(vec![tx.encode_to_vec()])
            .execute()
            .await?;
        let post_tx_snapshot = self.chain_b_ibc.storage.latest_snapshot();

        // validate the connection state is now "tryopen"
        {
            // Connection should not exist pre-commit
            assert!(pre_tx_snapshot
                .get_connection(&self.chain_b_ibc.connection_id)
                .await?
                .is_none(),);

            // Post-commit, the connection should be in the "tryopen" state.
            let connection = post_tx_snapshot
                .get_connection(&self.chain_b_ibc.connection_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no connection with the specified ID {} exists",
                        &self.chain_b_ibc.connection_id
                    )
                })?;

            assert_eq!(connection.state, ConnectionState::TryOpen);

            self.chain_b_ibc.connection = Some(connection);
        }

        self._sync_chains().await?;
        Ok(())
    }

    // sends a ConnectionOpenConfirm message to chain B
    // at this point, chain A is in OPEN and B is in TRYOPEN.
    // afterwards, chain A will be in OPEN and chain B will be in OPEN.
    pub async fn _build_and_send_connection_open_confirm(&mut self) -> Result<()> {
        self._sync_chains().await?;

        // https://github.com/penumbra-zone/hermes/blob/a34a11fec76de3b573b539c237927e79cb74ec00/crates/relayer/src/connection.rs#L1296
        let chain_b_connection_id = self.chain_b_ibc.connection_id.clone();
        let connection_of_b_on_a_response = self
            .chain_a_ibc
            .ibc_connection_query_client
            .connection(QueryConnectionRequest {
                connection_id: chain_b_connection_id.to_string(),
            })
            .await?
            .into_inner();

        // Build message(s) for updating client on destination
        println!("UPDATE4");
        let dst_client_target_height = self.chain_a_ibc.get_latest_height().await?;
        self._build_and_send_update_client_b(dst_client_target_height)
            .await?;

        let plan = {
            // This mocks the relayer constructing a connection open try message on behalf
            // of the counterparty chain.
            let ibc_msg = IbcRelay::ConnectionOpenConfirm(MsgConnectionOpenConfirm {
                conn_id_on_b: self.chain_b_ibc.connection_id.clone(),
                proof_conn_end_on_a: MerkleProof::decode(
                    connection_of_b_on_a_response.clone().proof.as_slice(),
                )?,
                proof_height_on_a: connection_of_b_on_a_response
                    .proof_height
                    .unwrap()
                    .try_into()?,
                signer: self.chain_a_ibc.signer.clone(),
            })
            .into();
            TransactionPlan {
                actions: vec![ibc_msg],
                // Now fill out the remaining parts of the transaction needed for verification:
                memo: None,
                detection_data: None, // We'll set this automatically below
                transaction_parameters: TransactionParameters {
                    chain_id: self.chain_b_ibc.chain_id.clone(),
                    ..Default::default()
                },
            }
        };
        let tx = self
            .chain_b_ibc
            .client()
            .await?
            .witness_auth_build(&plan)
            .await?;

        // Execute the transaction, applying it to the chain state.
        let pre_tx_snapshot = self.chain_b_ibc.storage.latest_snapshot();
        self.chain_b_ibc
            .node
            .block()
            .with_data(vec![tx.encode_to_vec()])
            .execute()
            .await?;
        let post_tx_snapshot = self.chain_b_ibc.storage.latest_snapshot();

        // validate the connection state is now "open"
        {
            // Connection should be in TRYOPEN pre-commit
            let connection = pre_tx_snapshot
                .get_connection(&self.chain_b_ibc.connection_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no connection with the specified ID {} exists",
                        &self.chain_b_ibc.connection_id
                    )
                })?;

            assert_eq!(connection.state, ConnectionState::TryOpen);

            // Post-commit, the connection should be in the "OPEN" state.
            let connection = post_tx_snapshot
                .get_connection(&self.chain_b_ibc.connection_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no connection with the specified ID {} exists",
                        &self.chain_b_ibc.connection_id
                    )
                })?;

            assert_eq!(connection.state, ConnectionState::Open);

            self.chain_b_ibc.connection = Some(connection);
        }

        Ok(())
    }
}

// tell chain A about chain B
async fn _build_and_send_update_client(
    chain_a_ibc: &mut TestNodeWithIBC,
    chain_b_ibc: &mut TestNodeWithIBC,
    target_height: Height,
) -> Result<()> {
    let chain_b_height = chain_b_ibc.get_latest_height().await?;
    println!("chain_b latest height: {:?}", chain_b_height);
    let chain_b_latest_block: penumbra_proto::util::tendermint_proxy::v1::GetBlockByHeightResponse =
        chain_b_ibc
            .tendermint_proxy_service_client
            .get_block_by_height(GetBlockByHeightRequest {
                height: chain_b_height.revision_height.try_into()?,
            })
            .await?
            .into_inner();

    // Look up the last recorded consensus state for the counterparty client on chain A
    // to determine the last trusted height.
    let client_state_of_b_on_a_response = chain_a_ibc
        .ibc_client_query_client
        .client_state(QueryClientStateRequest {
            client_id: chain_b_ibc.client_id.to_string(),
        })
        .await?
        .into_inner();
    let trusted_height = ibc_types::lightclients::tendermint::client_state::ClientState::try_from(
        client_state_of_b_on_a_response
            .clone()
            .client_state
            .unwrap(),
    )?
    .latest_height;
    println!(
        "Telling chain a about chain b latest block: {} and trusted height: {}",
        hex::encode(chain_b_latest_block.clone().block_id.unwrap().hash),
        trusted_height
    );
    // println!(
    //     "header: {:?}",
    //     chain_b_latest_block.block.clone().unwrap().header.unwrap()
    // );
    println!(
        "chain_id {}, height {}, last_commit_hash: {}",
        chain_b_latest_block
            .block
            .clone()
            .unwrap()
            .header
            .unwrap()
            .chain_id,
        chain_b_latest_block
            .block
            .clone()
            .unwrap()
            .header
            .unwrap()
            .height,
        hex::encode(
            chain_b_latest_block
                .block
                .clone()
                .unwrap()
                .header
                .unwrap()
                .last_commit_hash
        )
    );
    let plan = {
        let ibc_msg = IbcRelay::UpdateClient(MsgUpdateClient {
            signer: chain_b_ibc.signer.clone(),
            client_id: chain_a_ibc.client_id.clone(),
            client_message: chain_b_ibc
                // The TendermintHeader is derived from the Block
                // and represents chain B's claims about its current state.
                .create_tendermint_header(Some(trusted_height), chain_b_latest_block)?
                .into(),
        })
        .into();
        TransactionPlan {
            actions: vec![ibc_msg],
            // Now fill out the remaining parts of the transaction needed for verification:
            memo: None,
            detection_data: None, // We'll set this automatically below
            transaction_parameters: TransactionParameters {
                chain_id: chain_a_ibc.chain_id.clone(),
                ..Default::default()
            },
        }
    };
    let tx = chain_a_ibc
        .client()
        .await?
        .witness_auth_build(&plan)
        .await?;

    // Execute the transaction, applying it to the chain state.
    chain_a_ibc
        .node
        .block()
        .with_data(vec![tx.encode_to_vec()])
        .execute()
        .await?;

    // HERMES IMPL
    // let consensus_state = chain_b_ibc
    //     .ibc_client_query_client
    //     .consensus_state(QueryConsensusStateRequest {
    //         client_id: chain_a_ibc.client_id.to_string(),
    //         revision_number: target_height.revision_number,
    //         revision_height: target_height.revision_height,
    //         latest_height: false,
    //     })
    //     .await?
    //     .into_inner();

    // if let Some(consensus_state) = consensus_state.consensus_state {
    //     tracing::info!("consensus state already exists at height {target_height}, skipping update");
    //     tracing::trace!(?consensus_state, "consensus state");
    //     return Ok(());
    // }

    // let mut src_application_latest_height = chain_a_ibc.get_latest_height().await?;
    // // Wait for the source network to produce block(s) & reach `target_height`.
    // while src_application_latest_height < target_height {
    //     // advance both blocks
    //     chain_a_ibc.node.block().execute().await?;
    //     chain_b_ibc.node.block().execute().await?;
    //     src_application_latest_height = chain_a_ibc.get_latest_height().await?;
    // }

    // // Get the latest client state on destination.
    // let client_state_of_a_on_b_response = chain_b_ibc
    //     .ibc_client_query_client
    //     .client_state(QueryClientStateRequest {
    //         client_id: chain_a_ibc.client_id.to_string(),
    //     })
    //     .await?
    //     .into_inner();

    // let client_latest_height =
    //     ibc_types::lightclients::tendermint::client_state::ClientState::try_from(
    //         client_state_of_a_on_b_response
    //             .clone()
    //             .client_state
    //             .unwrap(),
    //     )?
    //     .latest_height;
    // let trusted_height = if client_latest_height < target_height {
    //     client_latest_height
    // } else {
    //     panic!("unsupported, no sending updates to the past");
    // };

    // if trusted_height >= target_height {
    //     tracing::warn!(
    //         "skipping update: trusted height ({}) >= chain target height ({})",
    //         trusted_height,
    //         target_height
    //     );

    //     return Ok(());
    // }

    // println!("target chain b height: {:?}", target_height);
    // let chain_b_latest_block: penumbra_proto::util::tendermint_proxy::v1::GetBlockByHeightResponse =
    //     chain_b_ibc
    //         .tendermint_proxy_service_client
    //         .get_block_by_height(GetBlockByHeightRequest {
    //             height: target_height.revision_height.try_into()?,
    //         })
    //         .await?
    //         .into_inner();

    // // Look up the last recorded consensus state for the counterparty client on chain A
    // // to determine the last trusted height.
    // // let prev_counterparty_consensus_state =
    // // ConsensusState::try_from(consensus_state.consensus_state.unwrap())?;
    // println!(
    //     "Telling chain a about chain b latest block: {}",
    //     hex::encode(chain_b_latest_block.clone().block_id.unwrap().hash)
    // );
    // // println!(
    // //     "header: {:?}",
    // //     chain_b_latest_block.block.clone().unwrap().header.unwrap()
    // // );
    // println!(
    //     "chain_id {}, height {}, last_commit_hash: {}",
    //     chain_b_latest_block
    //         .block
    //         .clone()
    //         .unwrap()
    //         .header
    //         .unwrap()
    //         .chain_id,
    //     chain_b_latest_block
    //         .block
    //         .clone()
    //         .unwrap()
    //         .header
    //         .unwrap()
    //         .height,
    //     hex::encode(
    //         chain_b_latest_block
    //             .block
    //             .clone()
    //             .unwrap()
    //             .header
    //             .unwrap()
    //             .last_commit_hash
    //     )
    // );
    // let plan = {
    //     let ibc_msg = IbcRelay::UpdateClient(MsgUpdateClient {
    //         signer: chain_b_ibc.signer.clone(),
    //         client_id: chain_a_ibc.client_id.clone(),
    //         client_message: chain_b_ibc
    //             // The TendermintHeader is derived from the Block
    //             // and represents chain B's claims about its current state.
    //             .create_tendermint_header(Some(trusted_height), chain_b_latest_block)?
    //             .into(),
    //     })
    //     .into();
    //     TransactionPlan {
    //         actions: vec![ibc_msg],
    //         // Now fill out the remaining parts of the transaction needed for verification:
    //         memo: None,
    //         detection_data: None, // We'll set this automatically below
    //         transaction_parameters: TransactionParameters {
    //             chain_id: chain_a_ibc.chain_id.clone(),
    //             ..Default::default()
    //         },
    //     }
    // };
    // let tx = chain_a_ibc
    //     .client()
    //     .await?
    //     .witness_auth_build(&plan)
    //     .await?;

    // // Execute the transaction, applying it to the chain state.
    // chain_a_ibc
    //     .node
    //     .block()
    //     .with_data(vec![tx.encode_to_vec()])
    //     .execute()
    //     .await?;
    Ok(())
}
