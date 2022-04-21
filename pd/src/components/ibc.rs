use std::convert::TryFrom;

use anyhow::Result;
use async_trait::async_trait;
use ibc::{
    clients::ics07_tendermint::{
        client_state::ClientState as TendermintClientState,
        consensus_state::ConsensusState as TendermintConsensusState,
        header::Header as TendermintHeader,
    },
    core::{
        ics02_client::{
            client_consensus::AnyConsensusState,
            client_state::{AnyClientState, ClientState},
            header::AnyHeader,
            height::Height,
            msgs::{create_client::MsgCreateAnyClient, update_client::MsgUpdateAnyClient},
        },
        ics24_host::identifier::ClientId,
    },
};
use penumbra_ibc::{ClientCounter, ClientData, ConsensusState, IBCAction, VerifiedHeights};
use penumbra_proto::ibc::ibc_action::Action::{CreateClient, UpdateClient};
use penumbra_transaction::{Action, Transaction};
use tendermint::{abci, Time};
use tendermint_light_client_verifier::{
    types::{Time as LightClientTime, TrustedBlockState, UntrustedBlockState},
    ProdVerifier, Verdict, Verifier,
};
use tracing::instrument;

use super::{app::View as _, Component};
use crate::{genesis, Overlay, OverlayExt};

pub struct IBCComponent {
    overlay: Overlay,
}

#[async_trait]
impl Component for IBCComponent {
    #[instrument(name = "ibc", skip(overlay))]
    async fn new(overlay: Overlay) -> Result<Self> {
        Ok(Self { overlay })
    }

    #[instrument(name = "ibc", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) -> Result<()> {
        // set the initial client count
        self.overlay.put_client_counter(ClientCounter(0)).await;

        Ok(())
    }

    #[instrument(name = "ibc", skip(self, _begin_block))]
    async fn begin_block(&mut self, _begin_block: &abci::request::BeginBlock) -> Result<()> {
        Ok(())
    }

    #[instrument(name = "ibc", skip(_tx))]
    fn check_tx_stateless(_tx: &Transaction) -> Result<()> {
        Ok(())
    }

    #[instrument(name = "ibc", skip(self, _tx))]
    async fn check_tx_stateful(&self, _tx: &Transaction) -> Result<()> {
        Ok(())
    }

    #[instrument(name = "ibc", skip(self, tx))]
    async fn execute_tx(&mut self, tx: &Transaction) -> Result<()> {
        // handle client transactions
        for ibc_action in tx
            .transaction_body
            .actions
            .iter()
            .filter_map(|action| match action {
                Action::IBCAction(ibc_action) => Some(ibc_action),
                _ => None,
            })
        {
            self.handle_ibc_action(ibc_action).await?;
        }

        Ok(())
    }

    #[instrument(name = "ibc", skip(self, _end_block))]
    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) -> Result<()> {
        Ok(())
    }
}

impl IBCComponent {
    async fn handle_ibc_action(&mut self, ibc_action: &IBCAction) -> Result<()> {
        match &ibc_action.action {
            CreateClient(raw_msg_create_client) => {
                // NOTE: MsgCreateAnyClient::try_from will validate that client_state and
                // consensus_state are Tendermint client and consensus states only, as these
                // are the only currently supported client types.
                let msg_create_client =
                    MsgCreateAnyClient::try_from(raw_msg_create_client.clone())?;

                self.handle_create_client(msg_create_client).await?;
            }
            UpdateClient(raw_msg_update_client) => {
                let msg_update_client =
                    MsgUpdateAnyClient::try_from(raw_msg_update_client.clone())?;

                self.handle_update_client(msg_update_client).await?;
            }
            _ => return Ok(()),
        }

        Ok(())
    }

    // Handle IBC UpdateClient. This is one of the most important IBC messages, as it has
    // the responsibility of verifying consensus state updates (through the semantics of
    // the light client's verification fn).
    //
    // verify:
    // - we have a client corresponding to the UpdateClient's client_id
    // - the stored client is not frozen
    // - the stored client is not expired
    // - the supplied update verifies using the semantics of the light client's header verification fn
    async fn handle_update_client(&mut self, msg_update_client: MsgUpdateAnyClient) -> Result<()> {
        // get the latest client state
        let client_data = self
            .overlay
            .get_client_data(&msg_update_client.client_id)
            .await?;

        // check that the client is not frozen or expired
        if client_data.client_state.0.is_frozen() {
            return Err(anyhow::anyhow!("client is frozen"));
        }
        // check if client is expired
        let latest_consensus_state = self
            .overlay
            .get_verified_consensus_state(
                client_data.client_state.0.latest_height(),
                client_data.client_id.clone(),
            )
            .await?;

        let latest_consensus_state_tm = match latest_consensus_state.0 {
            AnyConsensusState::Tendermint(consensus_state) => consensus_state,
            _ => {
                return Err(anyhow::anyhow!(
                    "consensus state is not a Tendermint client"
                ))
            }
        };
        let now = self.overlay.get_block_timestamp().await?;
        let stamp = latest_consensus_state_tm.timestamp.to_rfc3339();
        let duration = now.duration_since(Time::parse_from_rfc3339(&stamp).unwrap())?;
        if client_data.client_state.0.expired(duration) {
            return Err(anyhow::anyhow!("client is expired"));
        }

        // verify the clientupdate's header
        let tm_client_state = match client_data.clone().client_state.0 {
            AnyClientState::Tendermint(tm_state) => tm_state,
            _ => return Err(anyhow::anyhow!("unsupported client type")),
        };
        let tm_header = match msg_update_client.header {
            AnyHeader::Tendermint(tm_header) => tm_header,
            _ => {
                return Err(anyhow::anyhow!("client update is not a Tendermint header"));
            }
        };

        let (next_tm_client_state, next_tm_consensus_state) = self
            .verify_tendermint_update(
                msg_update_client.client_id.clone(),
                tm_client_state,
                tm_header.clone(),
            )
            .await?;

        // store the updated client and consensus states
        let height = self.overlay.get_block_height().await?;
        let next_client_data = client_data.with_new_client_state(
            AnyClientState::Tendermint(next_tm_client_state),
            now.to_rfc3339(),
            height,
        );
        self.overlay.put_client_data(next_client_data).await;
        self.overlay
            .put_verified_consensus_state(
                tm_header.height(),
                msg_update_client.client_id.clone(),
                ConsensusState(AnyConsensusState::Tendermint(next_tm_consensus_state)),
            )
            .await?;

        Ok(())
    }

    // Handle IBC CreateClient. Here we need to validate the following:
    // - client type is one of the supported types (currently, only Tendermint light clients)
    // - consensus state is valid (is of the same type as the client type, also currently only Tendermint consensus states are permitted)
    //
    // Then, we compute the client's ID (a concatenation of a monotonically increasing
    // integer, the number of clients on Penumbra, and the client type) and commit the
    // following to our state:
    // - client type
    // - consensus state
    // - processed time and height
    async fn handle_create_client(&mut self, msg_create_client: MsgCreateAnyClient) -> Result<()> {
        // get the current client counter
        let id_counter = self.overlay.client_counter().await?;
        let client_id = ClientId::new(msg_create_client.client_state.client_type(), id_counter.0)?;

        tracing::info!("creating client {:?}", client_id);

        let height = self.overlay.get_block_height().await?;
        let timestamp = self.overlay.get_block_timestamp().await?;

        let data = ClientData::new(
            client_id.clone(),
            msg_create_client.client_state,
            timestamp.to_rfc3339(),
            height,
        );

        // store the client data
        self.overlay.put_client_data(data.clone()).await;

        // store the genesis consensus state
        self.overlay
            .put_verified_consensus_state(
                data.client_state.0.latest_height(),
                client_id,
                ConsensusState(msg_create_client.consensus_state),
            )
            .await?;

        // increment client counter
        let counter = self
            .overlay
            .client_counter()
            .await
            .unwrap_or(ClientCounter(0));
        self.overlay
            .put_client_counter(ClientCounter(counter.0 + 1))
            .await;

        Ok(())
    }

    // verify a ClientUpdate for a Tendermint client
    async fn verify_tendermint_update(
        &self,
        client_id: ClientId,
        trusted_client_state: TendermintClientState,
        untrusted_header: TendermintHeader,
    ) -> Result<(TendermintClientState, TendermintConsensusState), anyhow::Error> {
        let untrusted_consensus_state = TendermintConsensusState::from(untrusted_header.clone());

        if untrusted_header.height().revision_number != trusted_client_state.chain_id.version() {
            return Err(anyhow::anyhow!(
                "client update revision number does not match client state"
            ));
        }

        // check if we already have a consensus state for this height, if we do, check that it is
        // the same as this update, if it is, return early. If we already have a consensus state
        // for this height, but it doesn't match the one provided in this update, verify the one in
        // this update and freeze the client.
        let mut conflicting_header: bool = false;
        match self
            .overlay
            .get_verified_consensus_state(untrusted_header.height(), client_id.clone())
            .await
        {
            Ok(stored_consensus_state) => match stored_consensus_state.0 {
                AnyConsensusState::Tendermint(stored_tm_consensus_state) => {
                    if stored_tm_consensus_state == untrusted_consensus_state {
                        return Ok((trusted_client_state, stored_tm_consensus_state));
                    } else {
                        conflicting_header = true;
                    }
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "stored consensus state is not a Tendermint state"
                    ));
                }
            },
            _ => {}
        }

        let last_trusted_consensus_state = match self
            .overlay
            .get_verified_consensus_state(untrusted_header.trusted_height, client_id.clone())
            .await?
            .0
        {
            AnyConsensusState::Tendermint(stored_tm_consensus_state) => stored_tm_consensus_state,
            _ => {
                return Err(anyhow::anyhow!(
                    "stored consensus state doesn't match client type"
                ))
            }
        };

        let trusted_state = TrustedBlockState {
            header_time: last_trusted_consensus_state.timestamp,
            height: untrusted_header
                .trusted_height
                .revision_height
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid header height"))?,
            next_validators: &untrusted_header.trusted_validator_set,
            next_validators_hash: last_trusted_consensus_state.next_validators_hash,
        };

        let untrusted_state = UntrustedBlockState {
            signed_header: &untrusted_header.signed_header,
            validators: &untrusted_header.validator_set,

            // TODO: how to we verify  next validator state?
            next_validators: None,
        };

        let options = trusted_client_state.as_light_client_options()?;
        let verifier = ProdVerifier::default();
        let current_block_timestamp = LightClientTime::parse_from_rfc3339(
            &self.overlay.get_block_timestamp().await?.to_rfc3339(),
        )
        .unwrap();
        let verdict = verifier.verify(
            untrusted_state,
            trusted_state,
            &options,
            current_block_timestamp,
        );
        match verdict {
            Verdict::Success => {}
            Verdict::NotEnoughTrust(voting_power_tally) => {
                return Err(anyhow::anyhow!(
                    "not enough trust, voting power tally: {:?}",
                    voting_power_tally
                ));
            }
            Verdict::Invalid(detail) => {
                return Err(anyhow::anyhow!(
                    "could not verify tendermint header: invalid: {:?}",
                    detail
                ));
            }
        }

        // consensus state is verified
        let verified_header = untrusted_header.clone();
        let verified_consensus_state = untrusted_consensus_state.clone();

        // if this header is verified, and conflicts with a past header that was previously
        // verified, we need to freeze the client.
        //
        // NOTE (divergence): we use the height of the header as the FrozenHeight. The Cosmos SDK
        // uses a static height, (0, 1), for all frozen clients.
        if conflicting_header {
            return Ok((
                trusted_client_state
                    .with_header(untrusted_header.clone())
                    .with_frozen_height(untrusted_header.height())?,
                verified_consensus_state,
            ));
        }

        // check that updates have monotonic timestamps. we may receive client updates that are
        // disjoint: the header we received and validated may be older than the newest header we
        // have. In that case, we need to verify that the timestamp is correct.
        let mut timestamp_monotonicity_violation = false;

        let next_consensus_state = self
            .overlay
            .next_verified_consensus_state(&client_id, verified_header.height())
            .await?;
        let prev_consensus_state = self
            .overlay
            .prev_verified_consensus_state(&client_id, verified_header.height())
            .await?;

        // case 1: if we have a verified consensus state previous to this header, verify that this
        // header's timestamp is greater than or equal to the stored consensus state's timestamp
        if let Some(prev_state) = prev_consensus_state {
            match prev_state.0 {
                AnyConsensusState::Tendermint(prev_state_tm) => {
                    if !(verified_header.signed_header.header().time >= prev_state_tm.timestamp) {
                        timestamp_monotonicity_violation = true;
                    }
                }
                _ => return Err(anyhow::anyhow!("unsupported consensus state type")),
            }
        }

        // case 2: if we have a verified consensus state with higher block height than this header,
        // verify that this header's timestamp is less than or equal to this header's timestamp.
        if let Some(next_state) = next_consensus_state {
            match next_state.0 {
                AnyConsensusState::Tendermint(next_state_tm) => {
                    if !(verified_header.signed_header.header().time <= next_state_tm.timestamp) {
                        timestamp_monotonicity_violation = true;
                    }
                }
                _ => return Err(anyhow::anyhow!("unsupported consensus state type")),
            }
        }

        // if we receive a verified header that has non-monotonic timestamps, we freeze the client
        // (following Cosmos SDK behavior here.)
        if timestamp_monotonicity_violation {
            return Ok((
                trusted_client_state
                    .with_header(verified_header.clone())
                    .with_frozen_height(verified_header.height())?,
                verified_consensus_state,
            ));
        }

        // TODO: pruning headers is possible here to improve performance.

        return Ok((
            trusted_client_state.with_header(untrusted_header.clone()),
            verified_consensus_state,
        ));
    }
}

#[async_trait]
pub trait View: OverlayExt + Send + Sync {
    async fn put_client_counter(&mut self, counter: ClientCounter) {
        self.put_domain("ibc/ics02-client/client_counter".into(), counter)
            .await;
    }
    async fn client_counter(&self) -> Result<ClientCounter> {
        self.get_domain("ibc/ics02-client/client_counter".into())
            .await
            .map(|counter| counter.unwrap_or(ClientCounter(0)))
    }
    async fn put_client_data(&mut self, data: ClientData) {
        self.put_domain(
            format!(
                "ibc/ics02-client/clients/{}",
                hex::encode(data.client_id.as_bytes())
            )
            .into(),
            data,
        )
        .await;
    }
    async fn get_client_data(&self, client_id: &ClientId) -> Result<ClientData> {
        let client_data = self
            .get_domain(
                format!(
                    "ibc/ics02-client/clients/{}",
                    hex::encode(client_id.as_bytes())
                )
                .into(),
            )
            .await?;

        client_data.ok_or(anyhow::anyhow!("client not found"))
    }

    async fn get_verified_heights(&self, client_id: &ClientId) -> Result<Option<VerifiedHeights>> {
        self.get_domain(
            format!(
                "ibc/ics02-client/clients/{}/verified_heights",
                hex::encode(client_id.as_bytes())
            )
            .into(),
        )
        .await
    }

    async fn put_verified_heights(
        &mut self,
        client_id: &ClientId,
        verified_heights: VerifiedHeights,
    ) {
        self.put_domain(
            format!(
                "ibc/ics02-client/clients/{}/verified_heights",
                hex::encode(client_id.as_bytes())
            )
            .into(),
            verified_heights,
        )
        .await;
    }

    async fn get_verified_consensus_state(
        &self,
        height: Height,
        client_id: ClientId,
    ) -> Result<ConsensusState> {
        self.get_domain(
            format!(
                "ibc/ics02-client/clients/{}/consensus_state/{}",
                hex::encode(client_id.as_bytes()),
                height
            )
            .into(),
        )
        .await?
        .ok_or(anyhow::anyhow!("consensus state not found"))
    }

    async fn put_verified_consensus_state(
        &mut self,
        height: Height,
        client_id: ClientId,
        consensus_state: ConsensusState,
    ) -> Result<()> {
        self.put_domain(
            format!(
                "ibc/ics02-client/clients/{}/consensus_state/{}",
                hex::encode(client_id.as_bytes()),
                height
            )
            .into(),
            consensus_state,
        )
        .await;

        // update verified heights
        let mut verified_heights =
            self.get_verified_heights(&client_id)
                .await?
                .unwrap_or(VerifiedHeights {
                    heights: Vec::new(),
                });

        verified_heights.heights.push(height.clone());

        self.put_verified_heights(&client_id, verified_heights)
            .await;

        Ok(())
    }

    // returns the lowest verified consensus state that is higher than the given height, if it
    // exists.
    async fn next_verified_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Option<ConsensusState>> {
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
                .get_verified_consensus_state(*next_height, client_id.clone())
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
        height: Height,
    ) -> Result<Option<ConsensusState>> {
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
                .get_verified_consensus_state(*prev_height, client_id.clone())
                .await?;
            return Ok(Some(prev_cons_state));
        } else {
            return Ok(None);
        }
    }
}

impl<T: OverlayExt + Send + Sync> View for T {}
