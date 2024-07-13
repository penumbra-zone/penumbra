use crate::terminal::{pretty_print_transaction_plan, read_password};
use anyhow::Error;
use iroh_net::endpoint::get_remote_node_id;
use iroh_net::key::{PublicKey, SecretKey};
use iroh_net::ticket::NodeTicket;
use iroh_net::Endpoint;
use penumbra_custody::threshold::{SigningRequest, Terminal};
use penumbra_keys::FullViewingKey;
use penumbra_proto::DomainType;
use quinn::{Connection, SendStream};
use serde::de::DeserializeOwned;
use std::collections::VecDeque;
use std::io;
use std::str::FromStr;
use std::sync::Arc;
use termion::{color, input::TermRead};
use tokio::sync::Mutex;
use tonic::async_trait;
use tracing::instrument;

pub const ALPN: &[u8] = b"PENUMBRATHRESHOLDV0";

pub const HANDSHAKE: [u8; 4] = *b"AQ=="; // (:

#[derive(Debug, Clone)]
pub enum Role {
    COORDINATOR,
    FOLLOWER,
}

/// An implementation of the threshold custody Terminal abstraction, which uses a
/// networked backend in order to allow threshold participants to directly coordinate.
#[derive(Debug, Clone)]
pub struct NetworkedTerminal {
    pub fvk: Option<FullViewingKey>,

    pub endpoint: Endpoint,

    pub coordinator: Arc<Mutex<Option<Connection>>>,
    pub followers: Arc<Mutex<Option<Vec<Connection>>>>,

    pub role: Role,
    pub dkg: bool,

    pub num_participants: u16,

    message_queue: Arc<Mutex<VecDeque<String>>>,
}

impl NetworkedTerminal {
    async fn needs_connect(&self) -> bool {
        let coordinator = self.coordinator.lock().await;
        let followers = self.followers.lock().await;

        coordinator.is_none() && followers.is_none()
    }
    pub async fn new(role: Role, dkg: bool, num_participants: u16) -> anyhow::Result<Self> {
        let ephemeral_network_key = SecretKey::generate();

        let endpoint = Endpoint::builder()
            .secret_key(ephemeral_network_key)
            .alpns(vec![ALPN.to_vec()])
            .bind(0)
            .await?;

        // wait for the endpoint to figure out its address before making a ticket
        while endpoint.home_relay().is_none() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        return Ok(Self {
            fvk: None,
            endpoint,
            coordinator: Arc::new(Mutex::new(None)),
            followers: Arc::new(Mutex::new(None)),
            role,
            dkg,
            num_participants,
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
        });
    }

    #[instrument]
    async fn connect(&self) -> anyhow::Result<()> {
        let node = self.endpoint.node_addr().await?;
        let mut short = node.clone();
        short.info.direct_addresses.clear();
        let short = NodeTicket::new(short)?;
        println!("This node's ticket: {}", short);

        match self.role {
            Role::COORDINATOR => {
                let mut followers = vec![];
                let mut follower_keys = vec![];
                for i in 0..self.num_participants - 1 {
                    println!(
                        "Enter follower ticket {}/{}: ",
                        i + 1,
                        self.num_participants - 1
                    );
                    let mut input = String::new();
                    let input = io::stdin()
                        .lock()
                        .read_line()
                        .expect("Failed to read line")
                        .map(|line| line.trim().to_string());
                    let input = input.ok_or(anyhow::anyhow!("empty ticket!"))?;
                    let ticket = NodeTicket::from_str(&input)?;
                    follower_keys.push(ticket.node_addr().node_id);
                }
                println!(
                    "Waiting for follower connections (give them the coordinator ticket, {})",
                    short
                );
                for i in 0..self.num_participants - 1 {
                    loop {
                        let Some(connecting) = self.endpoint.accept().await else {
                            break;
                        };
                        let connection = match connecting.await {
                            Ok(connection) => connection,
                            Err(cause) => {
                                tracing::warn!("error accepting connection: {}", cause);
                                // if accept fails, we want to continue accepting connections
                                continue;
                            }
                        };

                        // Authenticate the remote node
                        let remote_node_id = get_remote_node_id(&connection)?;
                        if !follower_keys.contains(&remote_node_id) {
                            return Err(anyhow::anyhow!(
                                "got a connection from an unauthenticated node"
                            ));
                        }

                        let (mut w, mut r) = connection.accept_bi().await?;
                        let mut buf = [0u8; HANDSHAKE.len()];
                        r.read_exact(&mut buf).await?;
                        anyhow::ensure!(buf == HANDSHAKE, "invalid handshake");

                        followers.push(connection);

                        println!(
                            "Connected to {}/{} followers",
                            i + 1,
                            self.num_participants - 1
                        );

                        break;
                    }
                }

                let mut f = self.followers.lock().await;
                *f = Some(followers);
            }

            Role::FOLLOWER => {
                println!("Enter a coordinator ticket: ");
                let input = io::stdin()
                    .lock()
                    .read_line()
                    .expect("Failed to read line")
                    .map(|line| line.trim().to_string())
                    .ok_or(anyhow::anyhow!("couldnt read input"))?;

                let ticket = NodeTicket::from_str(&input)?;

                println!("Connecting to coordinator...");

                loop {
                    let connection = match self
                        .endpoint
                        .connect(ticket.node_addr().clone(), ALPN)
                        .await
                    {
                        Ok(connection) => connection,
                        Err(cause) => {
                            println!("error accepting connection: {}", cause);
                            // if accept fails, we want to continue accepting connections
                            continue;
                        }
                    };

                    let (mut w, mut r) = connection.open_bi().await?;
                    w.write_all(&HANDSHAKE).await?;
                    let mut buf = [0u8; HANDSHAKE.len()];

                    let mut c = self.coordinator.lock().await;
                    *c = Some(connection);
                    break;
                }

                println!("Connected to coordinator.");
            }
        }

        return Ok(());
    }
}

#[async_trait]
impl Terminal for NetworkedTerminal {
    // We received a new transaction request.
    async fn confirm_request(&self, request: &SigningRequest) -> anyhow::Result<bool> {
        if self.needs_connect().await {
            self.connect().await?;
        }

        match request {
            SigningRequest::TransactionPlan(plan) => {
                pretty_print_transaction_plan(self.fvk.clone(), plan)?;
                println!("Do you approve this transaction?");
            }
            SigningRequest::ValidatorDefinition(def) => {
                println!("{}", serde_json::to_string_pretty(def)?);
                println!("Do you approve this validator definition?");
            }
            SigningRequest::ValidatorVote(vote) => {
                println!("{}", serde_json::to_string_pretty(vote)?);
                println!("Do you approve this validator vote?");
            }
        };

        println!("Press enter to continue");
        //        self.read_line_raw().await?;
        Ok(true)
    }

    fn explain(&self, msg: &str) -> anyhow::Result<()> {
        println!(
            "{}{}{}",
            color::Fg(color::Blue),
            msg,
            color::Fg(color::Reset)
        );
        Ok(())
    }

    async fn broadcast(&self, data: &str) -> anyhow::Result<()> {
        if self.needs_connect().await {
            self.connect().await?;
        }

        match self.role {
            Role::FOLLOWER => {
                let mut s = self
                    .coordinator
                    .lock()
                    .await
                    .clone()
                    .expect("should have coordinator")
                    .open_uni()
                    .await?;
                s.write_all(data.as_bytes()).await?;
                s.finish().await?;
            }
            Role::COORDINATOR => {
                for follower in self
                    .followers
                    .lock()
                    .await
                    .clone()
                    .expect("should have followers")
                {
                    let mut s = follower.open_uni().await?;
                    s.write_all(data.as_bytes()).await?;
                    s.finish().await?;
                }
            }
        }

        Ok(())
    }

    async fn read_line_raw(&self) -> anyhow::Result<String> {
        if self.needs_connect().await {
            self.connect().await?;
        }

        let res = match self.role {
            Role::FOLLOWER => {
                let mut r = self
                    .coordinator
                    .lock()
                    .await
                    .clone()
                    .expect("should have coordinator")
                    .accept_uni()
                    .await?;
                let b = r.read_to_end(163844).await?;

                let res = String::from_utf8(b)?;
                res
            }

            Role::COORDINATOR => {
                let mut queue = self.message_queue.lock().await;

                if let Some(message) = queue.pop_front() {
                    return Ok(message);
                }

                // Queue is empty, read from all followers
                let followers = self
                    .followers
                    .lock()
                    .await
                    .clone()
                    .expect("should have followers");

                for follower in followers.clone() {
                    let mut r = follower.accept_uni().await?;
                    let b = r.read_to_end(163844).await?;
                    if let Ok(message) = String::from_utf8(b) {
                        queue.push_back(message.clone());

                        // DKG requires all-to-all communication.
                        if self.dkg {
                            for f2 in followers.clone() {
                                if get_remote_node_id(&f2)? != get_remote_node_id(&follower)? {
                                    let mut s = f2.open_uni().await?;
                                    s.write_all(message.as_bytes()).await?;
                                    s.finish().await?;
                                }
                            }
                        }
                    }
                }

                // Return the first message or an error if none were read
                queue
                    .pop_front()
                    .ok_or_else(|| anyhow::anyhow!("No messages available from followers"))?
            }
        };

        Ok(res)
    }

    async fn get_password(&self) -> anyhow::Result<String> {
        read_password("Enter Password: ").await
    }
}
