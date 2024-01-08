use anyhow::{anyhow, Result};
use penumbra_keys::{keys::AddressIndex, Address, FullViewingKey};
use penumbra_proto::{
    custody::v1alpha1::{self as pb},
    DomainType,
};
use penumbra_transaction::{plan::TransactionPlan, AuthorizationData};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use tonic::{async_trait, Request, Response, Status};

use crate::AuthorizeRequest;

pub use self::config::Config;

mod config;
mod dkg;
mod sign;

fn to_json<T>(data: &T) -> Result<String>
where
    T: DomainType,
    anyhow::Error: From<<T as TryFrom<<T as DomainType>::Proto>>::Error>,
    <T as DomainType>::Proto: Serialize,
{
    Ok(serde_json::to_string(&data.to_proto())?)
}

fn from_json<'a, T: DomainType>(data: &'a str) -> Result<T>
where
    T: DomainType,
    anyhow::Error: From<<T as TryFrom<<T as DomainType>::Proto>>::Error>,
    <T as DomainType>::Proto: Deserialize<'a>,
{
    Ok(serde_json::from_str::<<T as DomainType>::Proto>(data)?.try_into()?)
}

/// A trait abstracting over the kind of terminal interface we expect.
///
/// This is mainly used to accomodate the kind of interaction we have with the CLI
/// interface, but it can also be plugged in with more general backends.
#[async_trait]
pub trait Terminal {
    /// Have a user confirm that they want to sign this transaction.
    ///
    /// In an actual terminal, this should display the transaction in a human readable
    /// form, and then get feedback from the user.
    async fn confirm_transaction(&self, transaction: &TransactionPlan) -> Result<bool>;

    /// Push an explanatory message to the terminal.
    ///
    /// This message has no relation to the actual protocol, it just allows explaining
    /// what subsequent data means, and what the user needs to do.
    ///
    /// Backends can replace this with a no-op.
    async fn explain(&self, msg: &str) -> Result<()>;

    /// Broadcast a message to other users.
    async fn broadcast(&self, data: &str) -> Result<()>;

    /// Wait for a response from *some* other user, it doesn't matter which.
    ///
    /// This function should not return None spuriously, when it does,
    /// it should continue to return None until a message is broadcast.
    async fn next_response(&self) -> Result<Option<String>>;
}

/// Act as a follower in the signing protocol.
///
/// All this function does is produce side effects on the terminal, potentially returning
/// early if the user on the other end did not want to sign the transaction.
pub async fn follow(config: &Config, terminal: &impl Terminal) -> Result<()> {
    // Round 1
    terminal
        .explain("Paste the coordinator's first message:")
        .await?;
    let round1_message: sign::CoordinatorRound1 = {
        let string = terminal
            .next_response()
            .await?
            .ok_or(anyhow!("expected message from coordinator"))?;
        from_json(&string)?
    };
    if !terminal.confirm_transaction(&round1_message.plan()).await? {
        return Ok(());
    }
    let (round1_reply, round1_state) = sign::follower_round1(&mut OsRng, config, round1_message)?;
    terminal
        .explain("Send this message to the coordinator:")
        .await?;
    terminal.broadcast(&to_json(&round1_reply)?).await?;
    // Round 2
    terminal
        .explain("Paste the coordinator's second message:")
        .await?;
    let round2_message: sign::CoordinatorRound2 = {
        let string = terminal
            .next_response()
            .await?
            .ok_or(anyhow!("expected message from coordinator"))?;
        from_json(&string)?
    };
    let round2_reply = sign::follower_round2(config, round1_state, round2_message)?;
    terminal
        .explain("Send this message to the coordinator:")
        .await?;
    terminal.broadcast(&to_json(&round2_reply)?).await?;

    Ok(())
}

/// A distributed key generation protocol, producing a config without a centralized dealer.
///
/// Unlike the deal method on Config, this method will never have any participant know
/// the key. Otherwise, the parameters controlling the threshold and the number of participants
/// are the same as that method.
///
/// This takes in a terminal, because it requires interacting with the other participants.
pub async fn dkg(t: u16, n: u16, terminal: &impl Terminal) -> Result<Config> {
    let expected_responses = n.saturating_sub(1) as usize;
    // Round 1 top
    let (round1_message, state) = dkg::round1(&mut OsRng, t, n)?;
    terminal
        .explain("Round 1/2: Send this message to all other participants:")
        .await?;
    terminal.broadcast(&to_json(&round1_message)?).await?;
    // Round 1 bottom
    terminal
        .explain(&format!(
            "Round 1/2: Gather {expected_responses} messages from the other participants:"
        ))
        .await?;
    let round1_replies = {
        let mut acc: Vec<dkg::Round1> = Vec::new();
        while acc.len() < expected_responses {
            let string = terminal
                .next_response()
                .await?
                .ok_or(anyhow!("expected message from another participant"))?;
            acc.push(from_json(&string)?);
        }
        acc
    };

    // Round 2 top
    let (round2_message, state) = dkg::round2(&mut OsRng, state, round1_replies)?;
    terminal
        .explain("Round 2/2: Send this message to all other participants:")
        .await?;
    terminal.broadcast(&to_json(&round2_message)?).await?;
    // Round 2 bottom
    terminal
        .explain(&format!(
            "Round 2/2: Gather {expected_responses} messages from the other participants:"
        ))
        .await?;
    let round2_replies = {
        let mut acc: Vec<dkg::Round2> = Vec::new();
        while acc.len() < expected_responses {
            let string = terminal
                .next_response()
                .await?
                .ok_or(anyhow!("expected message from another participant"))?;
            acc.push(from_json(&string)?);
        }
        acc
    };
    dkg::round3(&mut OsRng, state, round2_replies)
}

/// A custody backend using threshold signing.  
///
/// This backend is initialized with a full viewing key, but only a share
/// of the spend key, which is not enough to sign on its own. Instead,
/// other signers with the same type of configuration need to cooperate
/// to help produce a signature.
pub struct Threshold<T> {
    config: Config,
    terminal: T,
}

impl<T> Threshold<T> {
    pub fn new(config: Config, terminal: T) -> Self {
        Threshold { config, terminal }
    }
}

impl<T: Terminal> Threshold<T> {
    /// Try and create the necessary signatures to authorize the transaction plan.
    async fn authorize(&self, request: AuthorizeRequest) -> Result<AuthorizationData> {
        let plan = request.plan;

        // Round 1
        let (round1_message, state1) = sign::coordinator_round1(&mut OsRng, &self.config, plan)?;
        self.terminal
            .explain("Send this message to the other signers:")
            .await?;
        self.terminal.broadcast(&to_json(&round1_message)?).await?;
        self.terminal
            .explain(&format!(
                "Now, gather at least {} replies from the other signers, and paste them below:",
                self.config.threshold()
            ))
            .await?;
        let round1_replies = {
            let mut acc = Vec::new();
            // We need 1 less, since we've already included ourselves.
            for _ in 1..self.config.threshold() {
                let reply_str = self
                    .terminal
                    .next_response()
                    .await?
                    .ok_or(anyhow!("expected round1 reply"))?;
                let reply = from_json::<sign::FollowerRound1>(&reply_str)?;
                acc.push(reply);
            }
            acc
        };
        // Round 2
        let (round2_message, state2) =
            sign::coordinator_round2(&self.config, state1, &round1_replies)?;
        self.terminal
            .explain("Send this message to the other signers:")
            .await?;
        self.terminal.broadcast(&to_json(&round2_message)?).await?;
        self.terminal
            .explain(
                "Now, gather the replies from the *same* signers as Round 1, and paste them below:",
            )
            .await?;
        let round2_replies = {
            let mut acc = Vec::new();
            // We need 1 less, since we've already included ourselves.
            for _ in 1..self.config.threshold() {
                let reply_str = self
                    .terminal
                    .next_response()
                    .await?
                    .ok_or(anyhow!("expected round2 reply"))?;
                let reply = from_json::<sign::FollowerRound2>(&reply_str)?;
                acc.push(reply);
            }
            acc
        };
        // Round 3
        sign::coordinator_round3(&self.config, state2, &round2_replies)
    }

    /// Return the full viewing key.
    fn export_full_viewing_key(&self) -> FullViewingKey {
        self.config.fvk().clone()
    }

    /// Get the address associated with an index.
    ///
    /// This is just to match the API of the custody trait.
    fn confirm_address(&self, index: AddressIndex) -> Address {
        self.config.fvk().payment_address(index).0
    }
}

#[async_trait]
impl<T: Terminal + Sync + Send + 'static>
    pb::custody_protocol_service_server::CustodyProtocolService for Threshold<T>
{
    async fn authorize(
        &self,
        request: Request<pb::AuthorizeRequest>,
    ) -> Result<Response<pb::AuthorizeResponse>, Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("{e}")))?;
        let data = self.authorize(request).await.map_err(|e| {
            Status::internal(format!("Failed to process authorization request: {e}"))
        })?;
        Ok(Response::new(pb::AuthorizeResponse {
            data: Some(data.into()),
        }))
    }

    async fn export_full_viewing_key(
        &self,
        _request: Request<pb::ExportFullViewingKeyRequest>,
    ) -> Result<Response<pb::ExportFullViewingKeyResponse>, Status> {
        let fvk = self.export_full_viewing_key();
        Ok(Response::new(pb::ExportFullViewingKeyResponse {
            full_viewing_key: Some(fvk.into()),
        }))
    }

    async fn confirm_address(
        &self,
        request: Request<pb::ConfirmAddressRequest>,
    ) -> Result<Response<pb::ConfirmAddressResponse>, Status> {
        let index = request
            .into_inner()
            .address_index
            .ok_or(anyhow!("ConfirmAddressRequest missing address_index"))
            .and_then(|x| x.try_into())
            .map_err(|e| Status::invalid_argument(format!("{e}")))?;
        let address = self.confirm_address(index);
        Ok(Response::new(pb::ConfirmAddressResponse {
            address: Some(address.into()),
        }))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use tokio::sync;

    use super::*;

    struct FollowerTerminal {
        incoming: sync::Mutex<sync::mpsc::Receiver<String>>,
        outgoing: sync::mpsc::Sender<String>,
    }

    #[async_trait]
    impl Terminal for FollowerTerminal {
        async fn confirm_transaction(&self, _transaction: &TransactionPlan) -> Result<bool> {
            Ok(true)
        }

        async fn explain(&self, _msg: &str) -> Result<()> {
            Ok(())
        }

        async fn broadcast(&self, data: &str) -> Result<()> {
            self.outgoing.send(data.to_owned()).await?;
            Ok(())
        }

        async fn next_response(&self) -> Result<Option<String>> {
            Ok(self.incoming.lock().await.recv().await)
        }
    }

    struct CoordinatorTerminalInner {
        incoming: Vec<sync::mpsc::Receiver<String>>,
        i: usize,
    }

    impl CoordinatorTerminalInner {
        async fn recv(&mut self) -> Option<String> {
            let out = self.incoming[self.i].recv().await;
            self.i = (self.i + 1) % self.incoming.len();
            out
        }
    }

    struct CoordinatorTerminal {
        incoming: sync::Mutex<CoordinatorTerminalInner>,
        outgoing: Vec<sync::mpsc::Sender<String>>,
    }

    #[async_trait]
    impl Terminal for CoordinatorTerminal {
        async fn confirm_transaction(&self, _transaction: &TransactionPlan) -> Result<bool> {
            Ok(true)
        }

        async fn explain(&self, _msg: &str) -> Result<()> {
            Ok(())
        }

        async fn broadcast(&self, data: &str) -> Result<()> {
            for out in &self.outgoing {
                out.send(data.to_owned()).await?;
            }
            Ok(())
        }

        async fn next_response(&self) -> Result<Option<String>> {
            Ok(self.incoming.lock().await.recv().await)
        }
    }

    fn make_terminals(follower_count: usize) -> (CoordinatorTerminal, Vec<FollowerTerminal>) {
        let mut followers = Vec::new();
        let mut incoming = Vec::new();
        let mut outgoing = Vec::new();
        for _ in 0..follower_count {
            let (c2f_send, c2f_recv) = sync::mpsc::channel(1);
            let (f2c_send, f2c_recv) = sync::mpsc::channel(1);
            followers.push(FollowerTerminal {
                incoming: sync::Mutex::new(f2c_recv),
                outgoing: c2f_send,
            });
            incoming.push(c2f_recv);
            outgoing.push(f2c_send);
        }
        let coordinator = CoordinatorTerminal {
            incoming: sync::Mutex::new(CoordinatorTerminalInner { incoming, i: 0 }),
            outgoing,
        };
        (coordinator, followers)
    }

    fn make_symmetric_terminals(count: usize) -> Vec<CoordinatorTerminal> {
        // Make N^2 channels, ignore some of them:
        let mut sending = HashMap::new();
        let mut recving = HashMap::new();
        for i in 0..count {
            for j in 0..count {
                let (send, recv) = sync::mpsc::channel(1);
                sending.insert((i, j), send);
                recving.insert((i, j), recv);
            }
        }
        let mut out = Vec::new();
        for i in 0..count {
            let incoming = (0..count)
                .filter(|&j| j != i)
                .map(|j| recving.remove(&(j, i)).unwrap())
                .collect();
            let outgoing = (0..count)
                .filter(|&j| j != i)
                .map(|j| sending.remove(&(i, j)).unwrap())
                .collect();
            let coordinator = CoordinatorTerminal {
                incoming: sync::Mutex::new(CoordinatorTerminalInner { incoming, i: 0 }),
                outgoing,
            };
            out.push(coordinator);
        }
        out
    }

    async fn run_dkg(t: u16, n: u16) -> Result<Vec<Config>> {
        let terminals = make_symmetric_terminals(n as usize);
        let mut handles = Vec::new();
        for terminal in terminals {
            handles.push(tokio::spawn(async move { dkg(t, n, &terminal).await }));
        }
        let mut out = Vec::new();
        for handle in handles {
            out.push(handle.await??);
        }
        Ok(out)
    }

    #[tokio::test]
    async fn test_dkg_produces_identical_fvks() -> Result<()> {
        const T: u16 = 3;
        const N: u16 = 3;
        let (first_config, configs) = {
            let mut configs = run_dkg(T, N).await?;
            let first = configs.pop().unwrap();
            (first, configs)
        };
        for config in configs {
            assert_eq!(first_config.fvk(), config.fvk());
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_transaction_signing() -> Result<()> {
        const TEST_PLAN: &'static str = r#"
{
    "actions": [
        {
            "output": {
                "value": {
                    "amount": {
                        "lo": "1000000000"
                    },
                    "assetId": {
                        "inner": "KeqcLzNx9qSH5+lcJHBB9KNW+YPrBk5dKzvPMiypahA="
                    }
                },
                "destAddress": {
                    "inner": "UuFEV0VoZNxNTttsJVJzRqEzW4bm0z2RCxhUneve0KTvDjQipeg/1zx0ftbDjgr6uPiSA70yJIdlpFyxeLyXfAAtmSy6BCpR3YjEkf1bI5Q="
                },
                "rseed": "4m4bxumA0sHuonPjr12UnI4CWKj1wuq4y6rrMRb0nw0=",
                "valueBlinding": "HHS7tY19JuWMwdKJvtKs8AmhMVa7osSpZ+CCBszu/AE=",
                "proofBlindingR": "FmbXZoh5Pd2mEtiAEkkAZpllWo9pdwTPlXeODBXHUxA=",
                "proofBlindingS": "0x96kUchW8jFfnxglAoMtvzPT5/RLg2RvfkRKjlU8BA="
            }
        },
        {
            "spend": {
                "note": {
                    "value": {
                        "amount": {
                            "lo": "1000000000000"
                        },
                        "assetId": {
                            "inner": "KeqcLzNx9qSH5+lcJHBB9KNW+YPrBk5dKzvPMiypahA="
                        }
                    },
                    "rseed": "3svSxWREwvvVzb2upQuu3Cyr56O2kRbo0nuX4+OWcdc=",
                    "address": {
                        "inner": "6146pY5upA9bQa4tag+6hXpMXa2kO5fcicSJGVEUP4HhZt7m4FpwAJ3+qwr5gpbHUON7DigyEJRpeV31FATGdfJhHBzGDWC+CIvi8dyIzGo="
                    }
                },
                "position": "90",
                "randomizer": "dJvg8FGvw5rJAvtSQvlQ4imLXahVXn419+xroVMLSwA=",
                "valueBlinding": "Ce1/hBKLEMB/bjEA06b4zUJVEstNUjkDBWM3WrVu+QM=",
                "proofBlindingR": "gXA7M4VR48IoxKrf4w4jGae2O7OGlTecU/RBXd4g6QI=",
                "proofBlindingS": "7+Rhrve7mdgsKbkfFq41yfq9+Mx2qRAZDtwP3VUDAAs="
            }
        },
        {
            "output": {
                "value": {
                    "amount": {
                        "lo": "999000000000"
                    },
                    "assetId": {
                        "inner": "KeqcLzNx9qSH5+lcJHBB9KNW+YPrBk5dKzvPMiypahA="
                    }
                },
                "destAddress": {
                    "inner": "6146pY5upA9bQa4tag+6hXpMXa2kO5fcicSJGVEUP4HhZt7m4FpwAJ3+qwr5gpbHUON7DigyEJRpeV31FATGdfJhHBzGDWC+CIvi8dyIzGo="
                },
                "rseed": "rCTbPc6xWyEcDV73Pl+W6XXbACShVOM+8/vdc7RSLlo=",
                "valueBlinding": "DP0FN5CV4g9xZN6u2W6/4o6I/Zwr38n81q4YnJ6COAA=",
                "proofBlindingR": "KV3u8Dc+cZo0HFUIn7n95UkQVXWeYp+3vAVuIpCIZRI=",
                "proofBlindingS": "i00KyJVklWXUhVRy37N3p9szFIvo7383to/qxBexnBE="
            }
        }
    ],
    "transactionParameters": {
        "chainId": "penumbra-testnet-rhea-8b2dfc5c",
        "fee": {
            "amount": {}
        }
    },
    "detectionData": {
        "cluePlans": [
            {
                "address": {
                    "inner": "UuFEV0VoZNxNTttsJVJzRqEzW4bm0z2RCxhUneve0KTvDjQipeg/1zx0ftbDjgr6uPiSA70yJIdlpFyxeLyXfAAtmSy6BCpR3YjEkf1bI5Q="
                },
                "rseed": "1Li0Qx05txsyOrx2pfO9kD5rDSUMy9e+j/hHmucqARI="
            },
            {
                "address": {
                    "inner": "6146pY5upA9bQa4tag+6hXpMXa2kO5fcicSJGVEUP4HhZt7m4FpwAJ3+qwr5gpbHUON7DigyEJRpeV31FATGdfJhHBzGDWC+CIvi8dyIzGo="
                },
                "rseed": "ePtCm9/tFcpLBdlgyu8bYRKV5CHbqd823UGDhG1LsGY="
            }
        ]
    },
    "memo": {
        "plaintext": {
            "returnAddress": {
                "inner": "OB8AEHEehWo0o0/Dn7JtNmgdDX1VRPaDgn6MLl6n41hVjI3llljrTDCFRRjN5mkNwVwsAyJ/UdfjNIFzbGV62YVXfBJ/IMVTq2CNAHwR8Qo="
            }
        },
        "key": "3plOcPZzKKj8KT3sVdKnblUUFDRzCmMWYtgwB3BqfXQ="
    }
}
        "#;
        const T: u16 = 3;
        const N: u16 = 3;

        let (coordinator_config, follower_configs) = {
            let mut configs = run_dkg(T, N).await?;
            (configs.pop().unwrap(), configs)
        };
        let (coordinator_terminal, follower_terminals) = make_terminals((N - 1) as usize);
        for (config, terminal) in follower_configs
            .into_iter()
            .zip(follower_terminals.into_iter())
        {
            tokio::spawn(async move { follow(&config, &terminal).await });
        }
        let plan = serde_json::from_str::<TransactionPlan>(TEST_PLAN)?;
        let fvk = coordinator_config.fvk().clone();
        let authorization_data = Threshold::new(coordinator_config, coordinator_terminal)
            .authorize(AuthorizeRequest {
                plan: plan.clone(),
                pre_authorizations: Vec::new(),
            })
            .await?;
        assert_eq!(plan.effect_hash(&fvk), authorization_data.effect_hash);
        // The transaction plan only has spends
        for (randomizer, sig) in plan
            .spend_plans()
            .into_iter()
            .map(|x| x.randomizer)
            .zip(authorization_data.spend_auths)
        {
            fvk.spend_verification_key()
                .randomize(&randomizer)
                .verify(authorization_data.effect_hash.as_bytes(), &sig)?;
        }
        Ok(())
    }
}
