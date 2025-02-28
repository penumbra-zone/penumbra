use core::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use anyhow::{Context, Result};
use async_trait::async_trait;

use ibc_types::core::client::ClientId;
use ibc_types::core::client::ClientType;
use ibc_types::core::client::Height;

use ibc_types::path::{ClientConsensusStatePath, ClientStatePath, ClientTypePath};

use cnidarium::{StateRead, StateWrite};
use ibc_types::lightclients::tendermint::{
    client_state::ClientState as TendermintClientState,
    consensus_state::ConsensusState as TendermintConsensusState,
    header::Header as TendermintHeader,
};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

use crate::component::client_counter::{ClientCounter, VerifiedHeights};
use crate::prefix::MerklePrefixExt;
use crate::IBC_COMMITMENT_PREFIX;

use super::state_key;
use super::HostInterface;

/// ClientStatus represents the current status of an IBC client.
///
/// https://github.com/cosmos/ibc-go/blob/main/modules/core/exported/client.go#L30
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientStatus {
    /// Active is a status type of a client. An active client is allowed to be used.
    Active,
    /// Frozen is a status type of a client. A frozen client is not allowed to be used.
    Frozen,
    /// Expired is a status type of a client. An expired client is not allowed to be used.
    Expired,
    /// Unknown indicates there was an error in determining the status of a client.
    Unknown,
    /// Unauthorized indicates that the client type is not registered as an allowed client type.
    Unauthorized,
}

impl Display for ClientStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ClientStatus::Active => write!(f, "Active"),
            ClientStatus::Frozen => write!(f, "Frozen"),
            ClientStatus::Expired => write!(f, "Expired"),
            ClientStatus::Unknown => write!(f, "Unknown"),
            ClientStatus::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

#[async_trait]
pub(crate) trait Ics2ClientExt: StateWrite {
    // given an already verified tendermint header, and a trusted tendermint client state, compute
    // the next client and consensus states.
    async fn next_tendermint_state(
        &self,
        client_id: ClientId,
        trusted_client_state: TendermintClientState,
        verified_header: TendermintHeader,
    ) -> (TendermintClientState, TendermintConsensusState) {
        let verified_consensus_state = TendermintConsensusState::from(verified_header.clone());

        // if we have a stored consensus state for this height that conflicts, we need to freeze
        // the client. if it doesn't conflict, we can return early
        if let Ok(stored_cs_state) = self
            .get_verified_consensus_state(&verified_header.height(), &client_id)
            .await
        {
            if stored_cs_state == verified_consensus_state {
                return (trusted_client_state, verified_consensus_state);
            } else {
                return (
                    trusted_client_state
                        .with_header(verified_header.clone())
                        .expect("able to add header to client state")
                        .with_frozen_height(ibc_types::core::client::Height {
                            revision_number: 0,
                            revision_height: 1,
                        }),
                    verified_consensus_state,
                );
            }
        }

        // check that updates have monotonic timestamps. we may receive client updates that are
        // disjoint: the header we received and validated may be older than the newest header we
        // have. In that case, we need to verify that the timestamp is correct. if it isn't, freeze
        // the client.
        let next_consensus_state = self
            .next_verified_consensus_state(&client_id, &verified_header.height())
            .await
            .expect("able to get next verified consensus state");
        let prev_consensus_state = self
            .prev_verified_consensus_state(&client_id, &verified_header.height())
            .await
            .expect("able to get previous verified consensus state");

        // case 1: if we have a verified consensus state previous to this header, verify that this
        // header's timestamp is greater than or equal to the stored consensus state's timestamp
        if let Some(prev_state) = prev_consensus_state {
            if verified_header.signed_header.header().time < prev_state.timestamp {
                return (
                    trusted_client_state
                        .with_header(verified_header.clone())
                        .expect("able to add header to client state")
                        .with_frozen_height(ibc_types::core::client::Height {
                            revision_number: 0,
                            revision_height: 1,
                        }),
                    verified_consensus_state,
                );
            }
        }
        // case 2: if we have a verified consensus state with higher block height than this header,
        // verify that this header's timestamp is less than or equal to this header's timestamp.
        if let Some(next_state) = next_consensus_state {
            if verified_header.signed_header.header().time > next_state.timestamp {
                return (
                    trusted_client_state
                        .with_header(verified_header.clone())
                        .expect("able to add header to client state")
                        .with_frozen_height(ibc_types::core::client::Height {
                            revision_number: 0,
                            revision_height: 1,
                        }),
                    verified_consensus_state,
                );
            }
        }

        (
            trusted_client_state
                .with_header(verified_header.clone())
                .expect("able to add header to client state"),
            verified_consensus_state,
        )
    }
}

impl<T: StateWrite + ?Sized> Ics2ClientExt for T {}

#[async_trait]
pub trait ConsensusStateWriteExt: StateWrite + Sized {
    async fn put_verified_consensus_state<HI: HostInterface>(
        &mut self,
        height: Height,
        client_id: ClientId,
        consensus_state: TendermintConsensusState,
    ) -> Result<()> {
        self.put(
            IBC_COMMITMENT_PREFIX
                .apply_string(ClientConsensusStatePath::new(&client_id, &height).to_string()),
            consensus_state,
        );

        let current_height = HI::get_block_height(&self).await?;
        let revision_number = HI::get_revision_number(&self).await?;
        let current_time: ibc_types::timestamp::Timestamp =
            HI::get_block_timestamp(&self).await?.into();

        self.put_proto::<u64>(
            state_key::client_processed_times(&client_id, &height),
            current_time.nanoseconds(),
        );

        self.put(
            state_key::client_processed_heights(&client_id, &height),
            ibc_types::core::client::Height::new(revision_number, current_height)?,
        );

        // update verified heights
        let mut verified_heights =
            self.get_verified_heights(&client_id)
                .await?
                .unwrap_or(VerifiedHeights {
                    heights: Vec::new(),
                });

        verified_heights.heights.push(height);

        self.put_verified_heights(&client_id, verified_heights);

        Ok(())
    }
}

impl<T: StateWrite> ConsensusStateWriteExt for T {}

#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    fn put_client_counter(&mut self, counter: ClientCounter) {
        self.put("ibc_client_counter".into(), counter);
    }

    fn put_client(&mut self, client_id: &ClientId, client_state: TendermintClientState) {
        self.put_proto(
            IBC_COMMITMENT_PREFIX
                .apply_string(ibc_types::path::ClientTypePath(client_id.clone()).to_string()),
            ibc_types::lightclients::tendermint::client_type().to_string(),
        );

        self.put(
            IBC_COMMITMENT_PREFIX.apply_string(ClientStatePath(client_id.clone()).to_string()),
            client_state,
        );
    }

    fn put_verified_heights(&mut self, client_id: &ClientId, verified_heights: VerifiedHeights) {
        self.put(
            format!(
                // NOTE: this is an implementation detail of the Penumbra ICS2 implementation, so
                // it's not in the same path namespace.
                "penumbra_verified_heights/{client_id}/verified_heights"
            ),
            verified_heights,
        );
    }

    // returns the ConsensusState for the penumbra chain (this chain) at the given height
    fn put_penumbra_sdk_consensus_state(
        &mut self,
        height: Height,
        consensus_state: TendermintConsensusState,
    ) {
        // NOTE: this is an implementation detail of the Penumbra ICS2 implementation, so
        // it's not in the same path namespace.
        self.put(
            format!("penumbra_consensus_states/{height}"),
            consensus_state,
        );
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn client_counter(&self) -> Result<ClientCounter> {
        self.get("ibc_client_counter")
            .await
            .map(|counter| counter.unwrap_or(ClientCounter(0)))
    }

    async fn get_client_type(&self, client_id: &ClientId) -> Result<ClientType> {
        self.get_proto(
            &IBC_COMMITMENT_PREFIX.apply_string(ClientTypePath(client_id.clone()).to_string()),
        )
        .await?
        .context(format!("could not find client type for {client_id}"))
        .map(ClientType::new)
    }

    async fn get_client_state(&self, client_id: &ClientId) -> Result<TendermintClientState> {
        let client_state = self
            .get(
                &IBC_COMMITMENT_PREFIX.apply_string(ClientStatePath(client_id.clone()).to_string()),
            )
            .await?;

        client_state.context(format!("could not find client state for {client_id}"))
    }

    async fn get_client_status(
        &self,
        client_id: &ClientId,
        current_block_time: tendermint::Time,
    ) -> ClientStatus {
        let client_type = self.get_client_type(client_id).await;

        if client_type.is_err() {
            return ClientStatus::Unknown;
        }

        // let _client_type = client_type.expect("client type is Ok");
        // IBC-Go has a check here to see if the client type is allowed.
        // We don't have a similar allowlist in Penumbra, so we skip that check.
        // https://github.com/cosmos/ibc-go/blob/main/modules/core/02-client/types/params.go#L34

        let client_state = self.get_client_state(client_id).await;

        if client_state.is_err() {
            return ClientStatus::Unknown;
        }

        let client_state = client_state.expect("client state is Ok");

        if client_state.is_frozen() {
            return ClientStatus::Frozen;
        }

        // get latest consensus state to check for expiry
        let latest_consensus_state = self
            .get_verified_consensus_state(&client_state.latest_height(), client_id)
            .await;

        if latest_consensus_state.is_err() {
            // if the client state does not have an associated consensus state for its latest height
            // then it must be expired
            return ClientStatus::Expired;
        }

        let latest_consensus_state = latest_consensus_state.expect("latest consensus state is Ok");

        let time_elapsed = current_block_time.duration_since(latest_consensus_state.timestamp);
        if time_elapsed.is_err() {
            return ClientStatus::Unknown;
        }
        let time_elapsed = time_elapsed.expect("time elapsed is Ok");

        if client_state.expired(time_elapsed) {
            return ClientStatus::Expired;
        }

        ClientStatus::Active
    }

    async fn get_verified_heights(&self, client_id: &ClientId) -> Result<Option<VerifiedHeights>> {
        self.get(&format!(
            // NOTE: this is an implementation detail of the Penumbra ICS2 implementation, so
            // it's not in the same path namespace.
            "penumbra_verified_heights/{client_id}/verified_heights"
        ))
        .await
    }

    // returns the ConsensusState for the penumbra chain (this chain) at the given height
    async fn get_penumbra_sdk_consensus_state(
        &self,
        height: Height,
    ) -> Result<TendermintConsensusState> {
        // NOTE: this is an implementation detail of the Penumbra ICS2 implementation, so
        // it's not in the same path namespace.
        self.get(&format!("penumbra_consensus_states/{height}"))
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("penumbra consensus state not found for height {height}")
            })
    }

    async fn get_verified_consensus_state(
        &self,
        height: &Height,
        client_id: &ClientId,
    ) -> Result<TendermintConsensusState> {
        self.get(
            &IBC_COMMITMENT_PREFIX
                .apply_string(ClientConsensusStatePath::new(client_id, height).to_string()),
        )
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "counterparty consensus state not found for client {client_id} at height {height}"
            )
        })
    }

    async fn get_client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<ibc_types::core::client::Height> {
        self.get(&state_key::client_processed_heights(client_id, height))
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "client update time not found for client {client_id} at height {height}"
                )
            })
    }

    async fn get_client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<ibc_types::timestamp::Timestamp> {
        let timestamp_nanos = self
            .get_proto::<u64>(&state_key::client_processed_times(client_id, height))
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "client update time not found for client {client_id} at height {height}"
                )
            })?;

        ibc_types::timestamp::Timestamp::from_nanoseconds(timestamp_nanos)
            .context("invalid client update time")
    }

    // returns the lowest verified consensus state that is higher than the given height, if it
    // exists.
    async fn next_verified_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<TendermintConsensusState>> {
        let mut verified_heights =
            self.get_verified_heights(client_id)
                .await?
                .unwrap_or(VerifiedHeights {
                    heights: Vec::new(),
                });

        // WARNING: load-bearing sort
        verified_heights.heights.sort();

        if let Some(next_height) = verified_heights
            .heights
            .iter()
            .find(|&verified_height| verified_height > &height)
        {
            let next_cons_state = self
                .get_verified_consensus_state(next_height, client_id)
                .await?;
            return Ok(Some(next_cons_state));
        } else {
            return Ok(None);
        }
    }

    // returns the highest verified consensus state that is lower than the given height, if it
    // exists.
    async fn prev_verified_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<TendermintConsensusState>> {
        let mut verified_heights =
            self.get_verified_heights(client_id)
                .await?
                .unwrap_or(VerifiedHeights {
                    heights: Vec::new(),
                });

        // WARNING: load-bearing sort
        verified_heights.heights.sort();

        if let Some(prev_height) = verified_heights
            .heights
            .iter()
            .find(|&verified_height| verified_height < &height)
        {
            let prev_cons_state = self
                .get_verified_consensus_state(prev_height, client_id)
                .await?;
            return Ok(Some(prev_cons_state));
        } else {
            return Ok(None);
        }
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

#[cfg(test)]
mod tests {
    use base64::prelude::*;
    use std::sync::Arc;

    use super::*;
    use cnidarium::{ArcStateDeltaExt, StateDelta};
    use ibc_types::core::client::msgs::MsgUpdateClient;
    use ibc_types::{core::client::msgs::MsgCreateClient, DomainType};
    use penumbra_sdk_sct::component::clock::{EpochManager as _, EpochRead};
    use std::str::FromStr;
    use tendermint::Time;

    use crate::component::ibc_action_with_handler::IbcRelayWithHandlers;
    use crate::component::ClientStateReadExt;
    use crate::{IbcRelay, StateWriteExt};

    use crate::component::app_handler::{AppHandler, AppHandlerCheck, AppHandlerExecute};
    use ibc_types::core::channel::msgs::{
        MsgAcknowledgement, MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck,
        MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgRecvPacket, MsgTimeout,
    };

    struct MockHost {}

    #[async_trait]
    impl HostInterface for MockHost {
        async fn get_chain_id<S: StateRead>(_state: S) -> Result<String> {
            Ok("mock_chain_id".to_string())
        }

        async fn get_revision_number<S: StateRead>(_state: S) -> Result<u64> {
            Ok(0u64)
        }

        async fn get_block_height<S: StateRead>(state: S) -> Result<u64> {
            Ok(state.get_block_height().await?)
        }

        async fn get_block_timestamp<S: StateRead>(state: S) -> Result<tendermint::Time> {
            state.get_current_block_timestamp().await
        }
    }

    struct MockAppHandler {}

    #[async_trait]
    impl AppHandlerCheck for MockAppHandler {
        async fn chan_open_init_check<S: StateRead>(
            _state: S,
            _msg: &MsgChannelOpenInit,
        ) -> Result<()> {
            Ok(())
        }
        async fn chan_open_try_check<S: StateRead>(
            _state: S,
            _msg: &MsgChannelOpenTry,
        ) -> Result<()> {
            Ok(())
        }
        async fn chan_open_ack_check<S: StateRead>(
            _state: S,
            _msg: &MsgChannelOpenAck,
        ) -> Result<()> {
            Ok(())
        }
        async fn chan_open_confirm_check<S: StateRead>(
            _state: S,
            _msg: &MsgChannelOpenConfirm,
        ) -> Result<()> {
            Ok(())
        }
        async fn chan_close_confirm_check<S: StateRead>(
            _state: S,
            _msg: &MsgChannelCloseConfirm,
        ) -> Result<()> {
            Ok(())
        }
        async fn chan_close_init_check<S: StateRead>(
            _state: S,
            _msg: &MsgChannelCloseInit,
        ) -> Result<()> {
            Ok(())
        }
        async fn recv_packet_check<S: StateRead>(_state: S, _msg: &MsgRecvPacket) -> Result<()> {
            Ok(())
        }
        async fn timeout_packet_check<S: StateRead>(_state: S, _msg: &MsgTimeout) -> Result<()> {
            Ok(())
        }
        async fn acknowledge_packet_check<S: StateRead>(
            _state: S,
            _msg: &MsgAcknowledgement,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl AppHandlerExecute for MockAppHandler {
        async fn chan_open_init_execute<S: StateWrite>(_state: S, _msg: &MsgChannelOpenInit) {}
        async fn chan_open_try_execute<S: StateWrite>(_state: S, _msg: &MsgChannelOpenTry) {}
        async fn chan_open_ack_execute<S: StateWrite>(_state: S, _msg: &MsgChannelOpenAck) {}
        async fn chan_open_confirm_execute<S: StateWrite>(_state: S, _msg: &MsgChannelOpenConfirm) {
        }
        async fn chan_close_confirm_execute<S: StateWrite>(
            _state: S,
            _msg: &MsgChannelCloseConfirm,
        ) {
        }
        async fn chan_close_init_execute<S: StateWrite>(_state: S, _msg: &MsgChannelCloseInit) {}
        async fn recv_packet_execute<S: StateWrite>(_state: S, _msg: &MsgRecvPacket) -> Result<()> {
            Ok(())
        }
        async fn timeout_packet_execute<S: StateWrite>(_state: S, _msg: &MsgTimeout) -> Result<()> {
            Ok(())
        }
        async fn acknowledge_packet_execute<S: StateWrite>(
            _state: S,
            _msg: &MsgAcknowledgement,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl AppHandler for MockAppHandler {}

    // test that we can create and update a light client.
    #[tokio::test]
    async fn test_create_and_update_light_client() -> anyhow::Result<()> {
        use penumbra_sdk_sct::epoch::Epoch;
        // create a storage backend for testing

        // TODO(erwan): `apply_default_genesis` is not available here. We need a component
        // equivalent.
        let mut state = Arc::new(StateDelta::new(()));
        {
            // TODO: this is copied out of App::init_chain, can we put it somewhere else?
            let mut state_tx = state.try_begin_transaction().unwrap();
            state_tx.put_block_height(0);
            state_tx.put_epoch_by_height(
                0,
                Epoch {
                    index: 0,
                    start_height: 0,
                },
            );
            state_tx.put_epoch_by_height(
                1,
                Epoch {
                    index: 0,
                    start_height: 0,
                },
            );
            state_tx.apply();
        }

        // Light client verification is time-dependent.  In practice, the latest
        // (consensus) time will be delivered in each BeginBlock and written
        // into the state.  Here, set the block timestamp manually so it's
        // available to the unit test.
        let timestamp = Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z")?;
        let mut state_tx = state.try_begin_transaction().unwrap();
        state_tx.put_block_timestamp(1u64, timestamp);
        state_tx.put_block_height(1);
        state_tx.put_ibc_params(crate::params::IBCParameters {
            ibc_enabled: true,
            inbound_ics20_transfers_enabled: true,
            outbound_ics20_transfers_enabled: true,
        });
        state_tx.put_epoch_by_height(
            1,
            Epoch {
                index: 0,
                start_height: 0,
            },
        );
        state_tx.apply();

        // base64 encoded MsgCreateClient that was used to create the currently in-use Stargaze
        // light client on the cosmos hub:
        // https://cosmos.bigdipper.live/transactions/13C1ECC54F088473E2925AD497DDCC092101ADE420BC64BADE67D34A75769CE9
        let msg_create_client_stargaze_raw = BASE64_STANDARD
            .decode(include_str!("./test/create_client.msg").replace('\n', ""))
            .unwrap();
        let msg_create_stargaze_client =
            MsgCreateClient::decode(msg_create_client_stargaze_raw.as_slice()).unwrap();

        // base64 encoded MsgUpdateClient that was used to issue the first update to the in-use stargaze light client on the cosmos hub:
        // https://cosmos.bigdipper.live/transactions/24F1E19F218CAF5CA41D6E0B653E85EB965843B1F3615A6CD7BCF336E6B0E707
        let msg_update_client_stargaze_raw = BASE64_STANDARD
            .decode(include_str!("./test/update_client_1.msg").replace('\n', ""))
            .unwrap();
        let mut msg_update_stargaze_client =
            MsgUpdateClient::decode(msg_update_client_stargaze_raw.as_slice()).unwrap();

        msg_update_stargaze_client.client_id = ClientId::from_str("07-tendermint-0").unwrap();

        let create_client_action = IbcRelayWithHandlers::<MockAppHandler, MockHost>::new(
            IbcRelay::CreateClient(msg_create_stargaze_client),
        );
        let update_client_action = IbcRelayWithHandlers::<MockAppHandler, MockHost>::new(
            IbcRelay::UpdateClient(msg_update_stargaze_client),
        );

        create_client_action.check_stateless(()).await?;
        create_client_action.check_historical(state.clone()).await?;
        let mut state_tx = state.try_begin_transaction().unwrap();
        create_client_action
            .check_and_execute(&mut state_tx)
            .await?;
        state_tx.apply();

        // Check that state reflects +1 client apps registered.
        assert_eq!(state.client_counter().await.unwrap().0, 1);

        // Now we update the client and confirm that the update landed in state.
        update_client_action.check_stateless(()).await?;
        update_client_action.check_historical(state.clone()).await?;
        let mut state_tx = state.try_begin_transaction().unwrap();
        update_client_action
            .check_and_execute(&mut state_tx)
            .await?;
        state_tx.apply();

        // We've had one client update, yes. What about second client update?
        // https://cosmos.bigdipper.live/transactions/ED217D360F51E622859F7B783FEF98BDE3544AA32BBD13C6C77D8D0D57A19FFD
        let msg_update_second = BASE64_STANDARD
            .decode(include_str!("./test/update_client_2.msg").replace('\n', ""))
            .unwrap();

        let mut second_update = MsgUpdateClient::decode(msg_update_second.as_slice()).unwrap();
        second_update.client_id = ClientId::from_str("07-tendermint-0").unwrap();
        let second_update_client_action = IbcRelayWithHandlers::<MockAppHandler, MockHost>::new(
            IbcRelay::UpdateClient(second_update),
        );

        second_update_client_action.check_stateless(()).await?;
        second_update_client_action
            .check_historical(state.clone())
            .await?;
        let mut state_tx = state.try_begin_transaction().unwrap();
        second_update_client_action
            .check_and_execute(&mut state_tx)
            .await?;
        state_tx.apply();

        Ok(())
    }

    #[tokio::test]
    /// Check that we're not able to create a client if the IBC component is disabled.
    async fn test_disabled_ibc_component() -> anyhow::Result<()> {
        let mut state = Arc::new(StateDelta::new(()));
        let mut state_tx = state.try_begin_transaction().unwrap();
        state_tx.put_ibc_params(crate::params::IBCParameters {
            ibc_enabled: false,
            inbound_ics20_transfers_enabled: true,
            outbound_ics20_transfers_enabled: true,
        });

        let msg_create_client_stargaze_raw = BASE64_STANDARD
            .decode(include_str!("./test/create_client.msg").replace('\n', ""))
            .unwrap();
        let msg_create_stargaze_client =
            MsgCreateClient::decode(msg_create_client_stargaze_raw.as_slice()).unwrap();

        let create_client_action = IbcRelayWithHandlers::<MockAppHandler, MockHost>::new(
            IbcRelay::CreateClient(msg_create_stargaze_client),
        );
        state_tx.apply();

        create_client_action.check_stateless(()).await?;
        create_client_action
            .check_historical(state.clone())
            .await
            .expect_err("should not be able to create a client");

        Ok(())
    }
}
