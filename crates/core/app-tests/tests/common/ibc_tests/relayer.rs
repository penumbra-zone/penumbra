use {
    super::TestNodeWithIBC,
    anyhow::{anyhow, Result},
    ibc_proto::ibc::core::{
        channel::v1::{IdentifiedChannel, QueryChannelRequest, QueryConnectionChannelsRequest},
        client::v1::{QueryClientStateRequest, QueryConsensusStateRequest},
        connection::v1::QueryConnectionRequest,
    },
    ibc_types::{
        core::{
            channel::{
                channel::{Order, State as ChannelState},
                msgs::{
                    MsgAcknowledgement, MsgChannelOpenAck, MsgChannelOpenConfirm,
                    MsgChannelOpenInit, MsgChannelOpenTry, MsgRecvPacket,
                },
                packet::Sequence,
                ChannelId, IdentifiedChannelEnd, Packet, PortId, TimeoutHeight,
                Version as ChannelVersion,
            },
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
                ConnectionEnd, Counterparty, State as ConnectionState,
                Version as ConnectionVersion,
            },
        },
        lightclients::tendermint::{
            client_state::{AllowUpdate, ClientState as TendermintClientState},
            consensus_state::ConsensusState,
            header::Header as TendermintHeader,
            TrustThreshold,
        },
        timestamp::Timestamp,
        DomainType as _,
    },
    penumbra_sdk_asset::{asset::Cache, Value},
    penumbra_sdk_ibc::{
        component::{ChannelStateReadExt as _, ConnectionStateReadExt as _},
        IbcRelay, IbcToken, IBC_COMMITMENT_PREFIX, IBC_PROOF_SPECS,
    },
    penumbra_sdk_keys::keys::AddressIndex,
    penumbra_sdk_num::Amount,
    penumbra_sdk_proto::{util::tendermint_proxy::v1::GetBlockByHeightRequest, DomainType},
    penumbra_sdk_shielded_pool::{Ics20Withdrawal, OutputPlan, SpendPlan},
    penumbra_sdk_stake::state_key::chain,
    penumbra_sdk_transaction::{
        memo::MemoPlaintext, plan::MemoPlan, TransactionParameters, TransactionPlan,
    },
    prost::Message as _,
    rand::SeedableRng as _,
    rand_chacha::ChaCha12Core,
    sha2::Digest,
    std::{
        str::FromStr as _,
        time::{Duration, SystemTime, UNIX_EPOCH},
    },
    tendermint::{abci::Event, Time},
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

    pub async fn get_channel_states(&mut self) -> Result<(ChannelState, ChannelState)> {
        let channel_on_a_response = self
            .chain_a_ibc
            .ibc_channel_query_client
            .connection_channels(QueryConnectionChannelsRequest {
                connection: self.chain_a_ibc.connection_id.to_string(),
                pagination: None,
            })
            .await?
            .into_inner();
        let channel_on_b_response = self
            .chain_b_ibc
            .ibc_channel_query_client
            .connection_channels(QueryConnectionChannelsRequest {
                connection: self.chain_b_ibc.connection_id.to_string(),
                pagination: None,
            })
            .await?
            .into_inner();

        let channels_a = channel_on_a_response.channels;
        let channels_b = channel_on_b_response.channels;

        // Note: Mock relayer expects only a single channel per connection right now
        let channel_a_state = match channels_a.len() {
            0 => ChannelState::Uninitialized,
            _ => {
                let channel_a: IdentifiedChannelEnd = channels_a[0].clone().try_into().unwrap();
                channel_a.channel_end.state.try_into()?
            }
        };
        let channel_b_state = match channels_b.len() {
            0 => ChannelState::Uninitialized,
            _ => {
                let channel_b: IdentifiedChannelEnd = channels_b[0].clone().try_into().unwrap();
                channel_b.channel_end.state.try_into()?
            }
        };

        Ok((channel_a_state, channel_b_state))
    }

    /// Performs a connection handshake followed by a channel handshake
    /// between the two chains owned by the mock relayer.
    pub async fn _handshake(&mut self) -> Result<(), anyhow::Error> {
        // Perform connection handshake
        self._connection_handshake().await?;

        // Perform channel handshake
        self._channel_handshake().await?;

        // The two chains should now be able to perform IBC transfers
        // between each other.
        Ok(())
    }

    /// Establish a connection between the two chains owned by the mock relayer.
    pub async fn _connection_handshake(&mut self) -> Result<(), anyhow::Error> {
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
            tracing::info!("Send Connection Init to chain A");
            self._build_and_send_connection_open_init().await?;
        }

        let (a_state, b_state) = self.get_connection_states().await?;
        assert!(a_state == ConnectionState::Init && b_state == ConnectionState::Uninitialized);

        self._sync_chains().await?;

        // 2. send the OpenTry message to chain B
        {
            tracing::info!("send Connection OpenTry to chain B");
            self._build_and_send_connection_open_try().await?;
        }

        let (a_state, b_state) = self.get_connection_states().await?;
        assert!(a_state == ConnectionState::Init && b_state == ConnectionState::TryOpen);

        self._sync_chains().await?;

        // 3. Send the OpenAck message to chain A
        {
            tracing::info!("send Connection OpenAck to chain A");
            self._build_and_send_connection_open_ack().await?;
        }

        let (a_state, b_state) = self.get_connection_states().await?;
        assert!(a_state == ConnectionState::Open && b_state == ConnectionState::TryOpen);

        self._sync_chains().await?;

        // 4. Send the OpenConfirm message to chain B
        {
            tracing::info!("send Connection OpenConfirm to chain B");
            self._build_and_send_connection_open_confirm().await?;
        }

        let (a_state, b_state) = self.get_connection_states().await?;
        assert!(a_state == ConnectionState::Open && b_state == ConnectionState::Open);

        // Ensure the chain timestamps remain in sync
        self._sync_chains().await?;

        Ok(())
    }

    /// Establish a channel between the two chains owned by the mock relayer.
    pub async fn _channel_handshake(&mut self) -> Result<(), anyhow::Error> {
        // The IBC channel handshake has four steps (Init, Try, Ack, Confirm).
        // https://github.com/penumbra-zone/hermes/blob/a34a11fec76de3b573b539c237927e79cb74ec00/crates/relayer/src/channel.rs#L712
        // https://github.com/cosmos/ibc/blob/main/spec/core/ics-004-channel-and-packet-semantics/README.md

        self._sync_chains().await?;

        let (a_state, b_state) = self.get_channel_states().await?;
        assert!(a_state == ChannelState::Uninitialized && b_state == ChannelState::Uninitialized);

        // 1: send the Init message to chain A
        {
            tracing::info!("Send Channel Init to chain A");
            self._build_and_send_channel_open_init().await?;
        }

        let (a_state, b_state) = self.get_channel_states().await?;
        assert!(a_state == ChannelState::Init && b_state == ChannelState::Uninitialized);

        self._sync_chains().await?;

        // 2. send the OpenTry message to chain B
        {
            tracing::info!("send Channel OpenTry to chain B");
            self._build_and_send_channel_open_try().await?;
        }

        let (a_state, b_state) = self.get_channel_states().await?;
        assert!(a_state == ChannelState::Init && b_state == ChannelState::TryOpen);

        self._sync_chains().await?;

        // 3. Send the OpenAck message to chain A
        {
            tracing::info!("send Channel OpenAck to chain A");
            self._build_and_send_channel_open_ack().await?;
        }

        let (a_state, b_state) = self.get_channel_states().await?;
        assert!(a_state == ChannelState::Open && b_state == ChannelState::TryOpen);

        self._sync_chains().await?;

        // 4. Send the OpenConfirm message to chain B
        {
            tracing::info!("send Channel OpenConfirm to chain B");
            self._build_and_send_channel_open_confirm().await?;
        }

        let (a_state, b_state) = self.get_channel_states().await?;
        assert!(a_state == ChannelState::Open && b_state == ChannelState::Open);

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
                        // (root and substore, see [`penumbra_sdk_ibc::component::ics02_validation`]).
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
                version: Some(chain_a_ibc.connection_version.clone()),
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

    // helper function to build ChannelOpenTry to chain B
    pub async fn _build_and_send_channel_open_try(&mut self) -> Result<()> {
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

        let chain_b_height = self._build_and_send_update_client_a().await?;
        let chain_a_height = self._build_and_send_update_client_b().await?;

        let chan_end_on_a_response = self
            .chain_a_ibc
            .ibc_channel_query_client
            .channel(QueryChannelRequest {
                port_id: self.chain_a_ibc.port_id.to_string(),
                channel_id: self.chain_a_ibc.channel_id.to_string(),
            })
            .await?
            .into_inner();

        let proof_chan_end_on_a =
            MerkleProof::decode(chan_end_on_a_response.clone().proof.as_slice())?;

        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        self._sync_chains().await?;

        let proof_height_on_a: Height = chan_end_on_a_response
            .proof_height
            .clone()
            .unwrap()
            .try_into()?;

        self._build_and_send_update_client_b().await?;
        self._sync_chains().await?;

        let plan = {
            // This mocks the relayer constructing a channel open try message on behalf
            // of the counterparty chain.
            #[allow(deprecated)]
            let ibc_msg = IbcRelay::ChannelOpenTry(MsgChannelOpenTry {
                signer: self.chain_a_ibc.signer.clone(),
                port_id_on_b: self.chain_b_ibc.port_id.clone(),
                connection_hops_on_b: vec![self.chain_b_ibc.connection_id.clone()],
                port_id_on_a: self.chain_a_ibc.port_id.clone(),
                chan_id_on_a: self.chain_a_ibc.channel_id.clone(),
                version_supported_on_a: self.chain_a_ibc.channel_version.clone(),
                proof_chan_end_on_a,
                proof_height_on_a,
                // Ordering must be Unordered for ics20 transfer
                ordering: Order::Unordered,
                // Deprecated
                previous_channel_id: self.chain_a_ibc.channel_id.to_string(),
                // Deprecated: Only ics20 version is supported
                version_proposal: ChannelVersion::new("ics20-1".to_string()),
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

        // Validate the tx hash is what we expect:
        let tx_hash = sha2::Sha256::digest(&tx.encode_to_vec());

        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;

        // execute the transaction containing the opentry message
        self.chain_b_ibc
            .node
            .block()
            .with_data(vec![tx.encode_to_vec()])
            .execute()
            .await?;
        self.chain_b_ibc.node.block().execute().await?;
        let post_tx_snapshot = self.chain_b_ibc.storage.latest_snapshot();

        // validate the channel state is now "tryopen"
        {
            // Channel should not exist pre-commit
            assert!(pre_tx_snapshot
                .get_channel(&self.chain_b_ibc.channel_id, &self.chain_b_ibc.port_id)
                .await?
                .is_none(),);

            // Post-commit, the connection should be in the "tryopen" state.
            let channel = post_tx_snapshot
                .get_channel(&self.chain_b_ibc.channel_id, &self.chain_b_ibc.port_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no channel with the specified ID {} exists",
                        &self.chain_b_ibc.channel_id
                    )
                })?;

            assert_eq!(channel.state, ChannelState::TryOpen);

            self.chain_b_ibc.channel = Some(channel);
        }

        self._sync_chains().await?;

        Ok(())
    }

    // helper function to build ChannelOpenInit to chain A
    pub async fn _build_and_send_channel_open_init(&mut self) -> Result<()> {
        self._sync_chains().await?;
        let chain_a_ibc = &mut self.chain_a_ibc;
        let chain_b_ibc = &mut self.chain_b_ibc;

        let plan = {
            let ibc_msg = IbcRelay::ChannelOpenInit(MsgChannelOpenInit {
                signer: chain_b_ibc.signer.clone(),
                port_id_on_a: chain_a_ibc.port_id.clone(),
                connection_hops_on_a: vec![chain_b_ibc
                    .counterparty
                    .connection_id
                    .clone()
                    .expect("connection established")],
                port_id_on_b: chain_b_ibc.port_id.clone(),
                // ORdering must be unordered for Ics20 transfer
                ordering: Order::Unordered,
                // Only ics20 version is supported
                version_proposal: ChannelVersion::new("ics20-1".to_string()),
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
            // Channel should not exist pre-commit
            assert!(pre_tx_snapshot
                .get_channel(&chain_a_ibc.channel_id, &chain_a_ibc.port_id)
                .await?
                .is_none(),);

            // Post-commit, the channel should be in the "init" state.
            let channel = post_tx_snapshot
                .get_channel(&chain_a_ibc.channel_id, &chain_a_ibc.port_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no channel with the specified ID {} exists",
                        &chain_a_ibc.channel_id
                    )
                })?;

            assert_eq!(channel.state.clone(), ChannelState::Init);

            chain_a_ibc.channel = Some(channel.clone());
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
    pub async fn _build_and_send_update_client_b(&mut self) -> Result<Height> {
        tracing::info!(
            "send update client for chain {} to chain {}",
            self.chain_a_ibc.chain_id,
            self.chain_b_ibc.chain_id,
        );
        // reverse these because we're sending to chain B
        let chain_a_ibc = &mut self.chain_b_ibc;
        let chain_b_ibc = &mut self.chain_a_ibc;

        _build_and_send_update_client(chain_a_ibc, chain_b_ibc).await
    }

    // helper function to build UpdateClient to send to chain A
    pub async fn _build_and_send_update_client_a(&mut self) -> Result<Height> {
        tracing::info!(
            "send update client for chain {} to chain {}",
            self.chain_b_ibc.chain_id,
            self.chain_a_ibc.chain_id,
        );
        let chain_a_ibc = &mut self.chain_a_ibc;
        let chain_b_ibc = &mut self.chain_b_ibc;

        _build_and_send_update_client(chain_a_ibc, chain_b_ibc).await
    }

    // Send an ACK message to chain A
    pub async fn _build_and_send_channel_open_ack(&mut self) -> Result<()> {
        // This is a load-bearing block execution that should be removed
        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        self._sync_chains().await?;

        let chain_b_connection_id = self.chain_b_ibc.connection_id.clone();
        let chain_a_connection_id = self.chain_a_ibc.connection_id.clone();

        // Build message(s) for updating client on source
        let src_client_height = self._build_and_send_update_client_a().await?;
        // Build message(s) for updating client on destination
        let dst_client_height = self._build_and_send_update_client_b().await?;

        let chan_end_on_b_response = self
            .chain_b_ibc
            .ibc_channel_query_client
            .channel(QueryChannelRequest {
                port_id: self.chain_b_ibc.port_id.to_string(),
                channel_id: self.chain_b_ibc.channel_id.to_string(),
            })
            .await?
            .into_inner();

        let proof_height_on_b = chan_end_on_b_response
            .clone()
            .proof_height
            .expect("proof height should be present")
            .try_into()?;
        let proof_chan_end_on_b =
            MerkleProof::decode(chan_end_on_b_response.clone().proof.as_slice())?;

        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        self._build_and_send_update_client_a().await?;
        self._sync_chains().await?;

        let plan = {
            // This mocks the relayer constructing a channel open try message on behalf
            // of the counterparty chain.
            let ibc_msg = IbcRelay::ChannelOpenAck(MsgChannelOpenAck {
                port_id_on_a: self.chain_a_ibc.port_id.clone(),
                chan_id_on_a: self.chain_a_ibc.channel_id.clone(),
                chan_id_on_b: self.chain_b_ibc.channel_id.clone(),
                version_on_b: self.chain_b_ibc.channel_version.clone(),
                proof_chan_end_on_b,
                proof_height_on_b,
                signer: self.chain_b_ibc.signer.clone(),
            })
            .into();
            // let ibc_msg = IbcRelay::ChannelOpenAck(MsgChannelOpenAck::try_from(proto_ack)?).into();
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

        // validate the channel state is now "OPEN"
        {
            // Channel should be in INIT pre-commit
            let channel = pre_tx_snapshot
                .get_channel(&self.chain_a_ibc.channel_id, &self.chain_a_ibc.port_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no channel with the specified ID {} exists",
                        &self.chain_a_ibc.channel_id
                    )
                })?;

            assert_eq!(channel.state, ChannelState::Init);

            // Post-commit, the channel should be in the "OPEN" state.
            let channel = post_tx_snapshot
                .get_channel(&self.chain_a_ibc.channel_id, &self.chain_a_ibc.port_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no channelwith the specified ID {} exists",
                        &self.chain_a_ibc.channel_id
                    )
                })?;

            assert_eq!(channel.state, ChannelState::Open);

            self.chain_a_ibc.channel = Some(channel);
        }

        Ok(())
    }

    // Send an ACK message to chain A
    // https://github.com/penumbra-zone/hermes/blob/a34a11fec76de3b573b539c237927e79cb74ec00/crates/relayer/src/connection.rs#L1126
    pub async fn _build_and_send_connection_open_ack(&mut self) -> Result<()> {
        // This is a load-bearing block execution that should be removed
        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        self._sync_chains().await?;

        let chain_b_connection_id = self.chain_b_ibc.connection_id.clone();
        let chain_a_connection_id = self.chain_a_ibc.connection_id.clone();

        // Build message(s) for updating client on source
        let src_client_height = self._build_and_send_update_client_a().await?;
        // Build message(s) for updating client on destination
        let dst_client_height = self._build_and_send_update_client_b().await?;

        let connection_of_a_on_b_response = self
            .chain_b_ibc
            .ibc_connection_query_client
            .connection(QueryConnectionRequest {
                connection_id: chain_a_connection_id.to_string(),
            })
            .await?
            .into_inner();
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
        assert_eq!(
            connection_of_a_on_b_response.clone().proof_height,
            consensus_state_of_a_on_b_response.clone().proof_height
        );
        assert_eq!(
            client_state_of_a_on_b_response.clone().proof_height,
            consensus_state_of_a_on_b_response.clone().proof_height
        );

        let proof_height_on_b = client_state_of_a_on_b_response.clone().proof_height;

        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        self._build_and_send_update_client_a().await?;
        self._sync_chains().await?;

        let plan = {
            // This mocks the relayer constructing a connection open try message on behalf
            // of the counterparty chain.
            // we can't directly construct this because one of the struct fields is private
            // and it's not from this crate, but we _can_ create the proto type and then convert it!
            let proto_ack = ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck {
                connection_id: self.chain_a_ibc.connection_id.to_string(),
                counterparty_connection_id: chain_b_connection_id.to_string(),
                version: Some(ConnectionVersion::default().into()),
                client_state: Some(
                    client_state_of_a_on_b_response
                        .clone()
                        .client_state
                        .unwrap(),
                ),
                proof_height: Some(proof_height_on_b.unwrap()),
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

        let chain_b_height = self._build_and_send_update_client_a().await?;
        let chain_a_height = self._build_and_send_update_client_b().await?;

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
        let consensus_state_of_b_on_a_response = self
            .chain_a_ibc
            .ibc_client_query_client
            .consensus_state(QueryConsensusStateRequest {
                client_id: self.chain_b_ibc.client_id.to_string(),
                revision_number: 0,
                revision_height: 0,
                latest_height: true,
            })
            .await?
            .into_inner();

        // Then construct the ConnectionOpenTry message
        let proof_consensus_state_of_b_on_a =
            MerkleProof::decode(consensus_state_of_b_on_a_response.clone().proof.as_slice())?;

        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        self._sync_chains().await?;

        assert_eq!(
            consensus_state_of_b_on_a_response.proof_height,
            client_state_of_b_on_a_response.proof_height
        );
        assert_eq!(
            connection_of_b_on_a_response.proof_height,
            client_state_of_b_on_a_response.proof_height
        );

        let proofs_height_on_a: Height = connection_of_b_on_a_response
            .proof_height
            .clone()
            .unwrap()
            .try_into()?;

        let proof_client_state_of_b_on_a =
            MerkleProof::decode(client_state_of_b_on_a_response.clone().proof.as_slice())?;
        let proof_conn_end_on_a =
            MerkleProof::decode(connection_of_b_on_a_response.clone().proof.as_slice())?;
        let proof_consensus_state_of_b_on_a =
            MerkleProof::decode(consensus_state_of_b_on_a_response.clone().proof.as_slice())?;

        // TODO: too side-effecty?
        self.chain_b_ibc.counterparty.connection_id = Some(self.chain_a_ibc.connection_id.clone());
        self.chain_a_ibc.counterparty.connection_id = Some(self.chain_b_ibc.connection_id.clone());

        self._build_and_send_update_client_b().await?;
        self._sync_chains().await?;

        let cs: TendermintClientState = client_state_of_b_on_a_response
            .clone()
            .client_state
            .unwrap()
            .try_into()?;
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
                versions_on_a: vec![ConnectionVersion::default()],
                proof_conn_end_on_a,
                proof_client_state_of_b_on_a,
                proof_consensus_state_of_b_on_a,
                proofs_height_on_a,
                consensus_height_of_b_on_a: chain_b_height,
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

        // Validate the tx hash is what we expect:
        let tx_hash = sha2::Sha256::digest(&tx.encode_to_vec());

        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;

        // execute the transaction containing the opentry message
        self.chain_b_ibc
            .node
            .block()
            .with_data(vec![tx.encode_to_vec()])
            .execute()
            .await?;
        self.chain_b_ibc.node.block().execute().await?;
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
        // This is a load-bearing block execution that should be removed
        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
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

        let dst_client_target_height = self._build_and_send_update_client_b().await?;

        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        self._build_and_send_update_client_b().await?;
        self._sync_chains().await?;

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

    // sends a ChannelOpenConfirm message to chain B
    // at this point, chain A is in OPEN and B is in TRYOPEN.
    // afterwards, chain A will be in OPEN and chain B will be in OPEN.
    pub async fn _build_and_send_channel_open_confirm(&mut self) -> Result<()> {
        // This is a load-bearing block execution that should be removed
        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        self._sync_chains().await?;

        // https://github.com/penumbra-zone/hermes/blob/a34a11fec76de3b573b539c237927e79cb74ec00/crates/relayer/src/connection.rs#L1296
        let chan_end_on_a_response = self
            .chain_a_ibc
            .ibc_channel_query_client
            .channel(QueryChannelRequest {
                port_id: self.chain_a_ibc.port_id.to_string(),
                channel_id: self.chain_a_ibc.channel_id.to_string(),
            })
            .await?
            .into_inner();

        let dst_client_target_height = self._build_and_send_update_client_b().await?;

        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        self._build_and_send_update_client_b().await?;
        self._sync_chains().await?;

        let plan = {
            // This mocks the relayer constructing a channel open confirm message on behalf
            // of the counterparty chain.
            let ibc_msg = IbcRelay::ChannelOpenConfirm(MsgChannelOpenConfirm {
                proof_height_on_a: chan_end_on_a_response.proof_height.unwrap().try_into()?,
                signer: self.chain_a_ibc.signer.clone(),
                port_id_on_b: self.chain_b_ibc.port_id.clone(),
                chan_id_on_b: self.chain_b_ibc.channel_id.clone(),
                proof_chan_end_on_a: MerkleProof::decode(chan_end_on_a_response.proof.as_slice())?,
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

        // validate the channel state is now "open"
        {
            // Channel should be in TRYOPEN pre-commit
            let channel = pre_tx_snapshot
                .get_channel(&self.chain_b_ibc.channel_id, &self.chain_b_ibc.port_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no channel with the specified ID {} exists",
                        &self.chain_b_ibc.channel_id
                    )
                })?;

            assert_eq!(channel.state, ChannelState::TryOpen);

            // Post-commit, the channel should be in the "OPEN" state.
            let channel = post_tx_snapshot
                .get_channel(&self.chain_b_ibc.channel_id, &self.chain_b_ibc.port_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no channel with the specified ID {} exists",
                        &self.chain_b_ibc.channel_id
                    )
                })?;

            assert_eq!(channel.state, ChannelState::Open);

            self.chain_b_ibc.channel = Some(channel);
        }

        Ok(())
    }

    /// Sends an IBC transfer from chain A to chain B.
    ///
    /// Currently hardcoded to send 50% of the first note's value
    /// on chain A.
    pub async fn transfer_from_a_to_b(&mut self) -> Result<()> {
        // Ensure chain A has balance to transfer
        let chain_a_client = self.chain_a_ibc.client().await?;
        let chain_b_client = self.chain_b_ibc.client().await?;

        let chain_a_note = chain_a_client
            .notes
            .values()
            .cloned()
            .next()
            .ok_or_else(|| anyhow!("mock client had no note"))?;

        // Get the balance of that asset on chain A
        let pretransfer_balance_a: Amount = chain_a_client
            .spendable_notes_by_asset(chain_a_note.asset_id())
            .map(|n| n.value().amount)
            .sum();

        // Get the balance of that asset on chain B
        // The asset ID of the IBC transferred asset on chain B
        // needs to be computed.
        let asset_cache = Cache::with_known_assets();
        let denom = asset_cache
            .get(&chain_a_note.asset_id())
            .expect("asset ID should exist in asset cache")
            .clone();
        let ibc_token = IbcToken::new(
            &self.chain_b_ibc.channel_id,
            &self.chain_b_ibc.port_id,
            &denom.to_string(),
        );
        let pretransfer_balance_b: Amount = chain_b_client
            .spendable_notes_by_asset(ibc_token.id())
            .map(|n| n.value().amount)
            .sum();

        // We will transfer 50% of the `chain_a_note`'s value to the same address on chain B
        let transfer_value = Value {
            amount: (chain_a_note.amount().value() / 2).into(),
            asset_id: chain_a_note.asset_id(),
        };

        // Prepare and perform the transfer from chain A to chain B
        let destination_chain_address = chain_b_client.fvk.payment_address(AddressIndex::new(0)).0;
        let denom = asset_cache
            .get(&transfer_value.asset_id)
            .expect("asset ID should exist in asset cache")
            .clone();
        let amount = transfer_value.amount;
        // TODO: test timeouts
        // For this sunny path test, we'll set the timeouts very far in the future
        let timeout_height = Height {
            revision_height: 1_000_000,
            revision_number: 0,
        };
        // get the current time on the local machine
        let current_time_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;

        // add 2 days to current time
        let mut timeout_time = current_time_ns + 1.728e14 as u64;

        // round to the nearest 10 minutes
        timeout_time += 600_000_000_000 - (timeout_time % 600_000_000_000);

        let return_address = chain_a_client
            .fvk
            .ephemeral_address(
                rand_chacha::ChaChaRng::seed_from_u64(1312),
                AddressIndex::new(0),
            )
            .0;
        let withdrawal = Ics20Withdrawal {
            destination_chain_address: destination_chain_address.to_string(),
            denom,
            amount,
            timeout_height,
            timeout_time,
            return_address,
            // TODO: this is fine to hardcode for now but should ultimately move
            // to the mock relayer and be based on the handshake
            source_channel: ChannelId::from_str("channel-0")?,
            // Penumbra <-> Penumbra so false
            use_compat_address: false,
            use_transparent_address: false,
            ics20_memo: "".to_string(),
        };
        // There will need to be `Spend` and `Output` actions
        // within the transaction in order for it to balance
        let spend_plan = SpendPlan::new(
            &mut rand_chacha::ChaChaRng::seed_from_u64(1312),
            chain_a_note.clone(),
            chain_a_client
                .position(chain_a_note.commit())
                .expect("note should be in mock client's tree"),
        );
        let output_plan = OutputPlan::new(
            &mut rand_chacha::ChaChaRng::seed_from_u64(1312),
            // half the note is being withdrawn, so we can use `transfer_value` both for the withdrawal action
            // and the change output
            transfer_value.clone(),
            chain_a_client.fvk.payment_address(AddressIndex::new(0)).0,
        );

        let plan = {
            let ics20_msg = withdrawal.into();
            TransactionPlan {
                actions: vec![ics20_msg, spend_plan.into(), output_plan.into()],
                // Now fill out the remaining parts of the transaction needed for verification:
                memo: Some(MemoPlan::new(
                    &mut rand_chacha::ChaChaRng::seed_from_u64(1312),
                    MemoPlaintext::blank_memo(
                        chain_a_client.fvk.payment_address(AddressIndex::new(0)).0,
                    ),
                )),
                detection_data: None, // We'll set this automatically below
                transaction_parameters: TransactionParameters {
                    chain_id: self.chain_a_ibc.chain_id.clone(),
                    ..Default::default()
                },
            }
            .with_populated_detection_data(
                rand_chacha::ChaChaRng::seed_from_u64(1312),
                Default::default(),
            )
        };
        let tx = self
            .chain_a_ibc
            .client()
            .await?
            .witness_auth_build(&plan)
            .await?;

        let (_end_block_events, deliver_tx_events) = self
            .chain_a_ibc
            .node
            .block()
            .with_data(vec![tx.encode_to_vec()])
            .execute()
            .await?;
        self._sync_chains().await?;

        // Since multiple send_packet events can occur in a single deliver tx response,
        // we accumulate all the events and process them in a loop.
        let mut recv_tx_deliver_tx_events: Vec<Event> = Vec::new();
        // Now that the withdrawal has been processed on Chain A, the relayer
        // tells chain B to process the transfer. It does this by forwarding a
        // MsgRecvPacket to chain B.
        //
        // The relayer needs to extract the event that chain A emitted:
        for event in deliver_tx_events.iter() {
            if event.kind == "send_packet" {
                let mut packet_data_hex = None;
                let mut sequence = None;
                let mut port_on_a = None;
                let mut chan_on_a = None;
                let mut port_on_b = None;
                let mut chan_on_b = None;
                let mut timeout_height_on_b = None;
                let mut timeout_timestamp_on_b = None;
                for attr in &event.attributes {
                    match attr.key_str()? {
                        "packet_data_hex" => packet_data_hex = Some(attr.value_str()?.to_string()),
                        "packet_sequence" => sequence = Some(attr.value_str()?.to_string()),
                        "packet_src_port" => port_on_a = Some(attr.value_str()?.to_string()),
                        "packet_src_channel" => chan_on_a = Some(attr.value_str()?.to_string()),
                        "packet_dst_port" => port_on_b = Some(attr.value_str()?.to_string()),
                        "packet_dst_channel" => chan_on_b = Some(attr.value_str()?.to_string()),
                        "packet_timeout_height" => {
                            timeout_height_on_b = Some(attr.value_str()?.to_string())
                        }
                        "packet_timeout_timestamp" => {
                            timeout_timestamp_on_b = Some(attr.value_str()?.to_string())
                        }
                        _ => (),
                    }
                }

                let port_on_a = port_on_a.expect("port_on_a attribute should be present");
                let chan_on_a = chan_on_a.expect("chan_on_a attribute should be present");
                let port_on_b = port_on_b.expect("port_on_b attribute should be present");
                let chan_on_b = chan_on_b.expect("chan_on_b attribute should be present");
                let sequence = sequence.expect("sequence attribute should be present");
                let timeout_height_on_b =
                    timeout_height_on_b.expect("timeout_height_on_b attribute should be present");
                let timeout_timestamp_on_b = timeout_timestamp_on_b
                    .expect("timeout_timestamp_on_b attribute should be present");
                let packet_data_hex =
                    packet_data_hex.expect("packet_data_hex attribute should be present");

                // The relayer must fetch the packet commitment proof from chain A
                // to include in the MsgRecvPacket
                // For a real relayer this would be done with an abci request, but
                // since we don't have a real cometbft node, we will just grab it
                // from storage
                let chain_a_snapshot = self.chain_a_ibc.storage.latest_snapshot();
                let (_commitment, proof_commitment_on_a) = chain_a_snapshot.get_with_proof(format!("ibc-data/commitments/ports/{port_on_a}/channels/{chan_on_a}/sequences/{sequence}").as_bytes().to_vec()).await?;

                // Now update the chains
                let _chain_b_height = self._build_and_send_update_client_a().await?;
                let chain_a_height = self._build_and_send_update_client_b().await?;

                let proof_height = chain_a_height;

                let msg_recv_packet = MsgRecvPacket {
                    packet: Packet {
                        sequence: Sequence::from_str(&sequence)?,
                        port_on_a: PortId::from_str(&port_on_a)?,
                        chan_on_a: ChannelId::from_str(&chan_on_a)?,
                        port_on_b: PortId::from_str(&port_on_b)?,
                        chan_on_b: ChannelId::from_str(&chan_on_b)?,
                        data: hex::decode(packet_data_hex)?,
                        timeout_height_on_b: TimeoutHeight::from_str(&timeout_height_on_b)?,
                        timeout_timestamp_on_b: Timestamp::from_str(&timeout_timestamp_on_b)?,
                    },
                    proof_commitment_on_a,
                    proof_height_on_a: Height {
                        revision_height: proof_height.revision_height,
                        revision_number: 0,
                    },
                    signer: self.chain_a_ibc.signer.clone(),
                };

                let plan = {
                    let ics20_msg = penumbra_sdk_transaction::ActionPlan::IbcAction(
                        IbcRelay::RecvPacket(msg_recv_packet),
                    )
                    .into();
                    TransactionPlan {
                        actions: vec![ics20_msg],
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

                let (_end_block_events, dtx_events) = self
                    .chain_b_ibc
                    .node
                    .block()
                    .with_data(vec![tx.encode_to_vec()])
                    .execute()
                    .await?;
                recv_tx_deliver_tx_events.extend(dtx_events.0.into_iter());
            }
        }

        self._sync_chains().await?;

        // Now that the transfer packet has been processed by chain B,
        // the relayer tells chain A to process the acknowledgement.
        for event in recv_tx_deliver_tx_events.iter() {
            if event.kind == "write_acknowledgement" {
                let mut packet_data_hex = None;
                let mut sequence = None;
                let mut port_on_a = None;
                let mut chan_on_a = None;
                let mut port_on_b = None;
                let mut chan_on_b = None;
                let mut timeout_height_on_b = None;
                let mut timeout_timestamp_on_b = None;
                let mut packet_ack_hex = None;
                for attr in &event.attributes {
                    match attr.key_str()? {
                        "packet_data_hex" => packet_data_hex = Some(attr.value_str()?.to_string()),
                        "packet_sequence" => sequence = Some(attr.value_str()?.to_string()),
                        "packet_src_port" => port_on_a = Some(attr.value_str()?.to_string()),
                        "packet_src_channel" => chan_on_a = Some(attr.value_str()?.to_string()),
                        "packet_dst_port" => port_on_b = Some(attr.value_str()?.to_string()),
                        "packet_dst_channel" => chan_on_b = Some(attr.value_str()?.to_string()),
                        "packet_timeout_height" => {
                            timeout_height_on_b = Some(attr.value_str()?.to_string())
                        }
                        "packet_timeout_timestamp" => {
                            timeout_timestamp_on_b = Some(attr.value_str()?.to_string())
                        }
                        "packet_ack_hex" => packet_ack_hex = Some(attr.value_str()?.to_string()),
                        _ => (),
                    }
                }

                let port_on_a = port_on_a.expect("port_on_a attribute should be present");
                let chan_on_a = chan_on_a.expect("chan_on_a attribute should be present");
                let port_on_b = port_on_b.expect("port_on_b attribute should be present");
                let chan_on_b = chan_on_b.expect("chan_on_b attribute should be present");
                let sequence = sequence.expect("sequence attribute should be present");
                let timeout_height_on_b =
                    timeout_height_on_b.expect("timeout_height_on_b attribute should be present");
                let timeout_timestamp_on_b = timeout_timestamp_on_b
                    .expect("timeout_timestamp_on_b attribute should be present");
                let packet_data_hex =
                    packet_data_hex.expect("packet_data_hex attribute should be present");
                let packet_ack_hex =
                    packet_ack_hex.expect("packet_ack_hex attribute should be present");

                let chain_b_snapshot = self.chain_b_ibc.storage.latest_snapshot();
                let (_commitment, proof_acked_on_b) = chain_b_snapshot
                    .get_with_proof(
                        format!(
                        "ibc-data/acks/ports/{port_on_b}/channels/{chan_on_b}/sequences/{sequence}"
                    )
                        .as_bytes()
                        .to_vec(),
                    )
                    .await?;

                // Now update the chains
                let _chain_a_height = self._build_and_send_update_client_b().await?;
                let chain_b_height = self._build_and_send_update_client_a().await?;

                let proof_height = chain_b_height;

                let msg_ack = MsgAcknowledgement {
                    signer: self.chain_a_ibc.signer.clone(),
                    packet: Packet {
                        sequence: Sequence::from_str(&sequence)?,
                        port_on_a: PortId::from_str(&port_on_a)?,
                        chan_on_a: ChannelId::from_str(&chan_on_a)?,
                        port_on_b: PortId::from_str(&port_on_b)?,
                        chan_on_b: ChannelId::from_str(&chan_on_b)?,
                        data: hex::decode(packet_data_hex)?,
                        timeout_height_on_b: TimeoutHeight::from_str(&timeout_height_on_b)?,
                        timeout_timestamp_on_b: Timestamp::from_str(&timeout_timestamp_on_b)?,
                    },
                    acknowledgement: hex::decode(packet_ack_hex)?,
                    proof_acked_on_b,
                    proof_height_on_b: Height {
                        revision_height: proof_height.revision_height,
                        revision_number: 0,
                    },
                };

                let plan = {
                    let ics20_msg = penumbra_sdk_transaction::ActionPlan::IbcAction(
                        IbcRelay::Acknowledgement(msg_ack),
                    )
                    .into();
                    TransactionPlan {
                        actions: vec![ics20_msg],
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

                self.chain_a_ibc
                    .node
                    .block()
                    .with_data(vec![tx.encode_to_vec()])
                    .execute()
                    .await?;
            }
        }

        self.chain_a_ibc.node.block().execute().await?;
        self.chain_b_ibc.node.block().execute().await?;
        self._sync_chains().await?;

        Ok(())
    }
}

// tell chain A about chain B. returns the height of chain b on chain a after update.
async fn _build_and_send_update_client(
    chain_a_ibc: &mut TestNodeWithIBC,
    chain_b_ibc: &mut TestNodeWithIBC,
) -> Result<Height> {
    let chain_b_height = chain_b_ibc.get_latest_height().await?;
    let chain_b_latest_block: penumbra_sdk_proto::util::tendermint_proxy::v1::GetBlockByHeightResponse =
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
    let chain_b_new_height = chain_b_latest_block
        .block
        .clone()
        .unwrap()
        .header
        .unwrap()
        .height;
    let plan = {
        let ibc_msg = IbcRelay::UpdateClient(MsgUpdateClient {
            signer: chain_b_ibc.signer.clone(),
            client_id: chain_a_ibc.client_id.clone(),
            client_message: chain_b_ibc
                // The TendermintHeader is derived from the Block
                // and represents chain B's claims about its current state.
                .create_tendermint_header(Some(trusted_height), chain_b_latest_block.clone())?
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

    Ok(Height {
        revision_height: chain_b_new_height as u64,
        revision_number: 0,
    })
}
