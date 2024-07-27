use std::collections::VecDeque;
use std::io;
use std::sync::Arc;
use magic_wormhole::transfer::APP_CONFIG;
use magic_wormhole::{MailboxConnection, Wormhole};
use termion::input::TermRead;
use tokio::sync::Mutex;
use tonic::async_trait;
use tracing::instrument;

use penumbra_custody::threshold::{SigningRequest, Terminal};
use penumbra_keys::FullViewingKey;

use crate::terminal::{pretty_print_transaction_plan, read_password};


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

    pub coordinator: Arc<Mutex<Option<Wormhole>>>,
    pub followers: Arc<Mutex<Option<Vec<Wormhole>>>>,

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
        return Ok(Self {
            fvk: None,
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
        // we use the same default app config, with the same default app id, in order to increase privacy.
        let config = APP_CONFIG.clone();

        match self.role {
            Role::COORDINATOR => {
                println!("Running as coordinator.");

                let mut followers = vec![];
                for i in 0..self.num_participants - 1 {
                    let mailbox = MailboxConnection::create(config.clone(), 2).await?;
                    println!("Waiting for follower {}/{} to connect, use ticket: {}", i+1, self.num_participants-1, mailbox.code().clone());
                    let connection = Wormhole::connect(mailbox).await?;
                    followers.push(connection);
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

                println!("Connecting to coordinator...");

                let code = magic_wormhole::Code(input);
                let mailbox = MailboxConnection::connect(config.clone(), code, false).await?;
                let connection = Wormhole::connect(mailbox).await?;

                println!("Connected to coordinator.");
                let mut c = self.coordinator.lock().await;
                *c = Some(connection);
            }
        }

        if self.dkg {
            println!("Connected! Running FROST dkg...");
        } else {
            println!("Connected! Running FROST signing...");
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
        io::stdin().lock().read_line()?;
        Ok(true)
    }

    fn explain(&self, _: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn broadcast(&self, data: &str) -> anyhow::Result<()> {
        if self.needs_connect().await {
            self.connect().await?;
        }

        match self.role {
            Role::FOLLOWER => {
                let mut s = self.coordinator.lock().await;
                let s = s.as_mut().expect("should have coordinator");

                s.send(data.as_bytes().to_vec()).await?;
                println!("Sent {} bytes to coordinator", data.len())
            }
            Role::COORDINATOR => {
                for follower in self
                    .followers
                    .lock()
                    .await
                    .as_mut()
                    .expect("should have followers")
                {
                    follower.send(data.as_bytes().to_vec()).await?;
                    println!("Sent {} bytes to follower", data.len());
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
                let mut coordinator_guard = self.coordinator.lock().await;
                let r = coordinator_guard.as_mut().expect("should have coordinator");
                let b = r.receive().await?;

                println!("Received {} bytes from coordinator", b.len());

                let res = String::from_utf8(b)?;
                res
            }

            Role::COORDINATOR => {
                let mut queue = self.message_queue.lock().await;

                if let Some(message) = queue.pop_front() {
                    return Ok(message);
                }

                // Queue is empty, read from all followers
                let mut follower_guard = self.followers.lock().await;
                let followers = follower_guard.as_mut().expect("should have followers");

                for i1 in 0..followers.len() {
                    let b = {
                        let follower = &mut followers[i1];
                        follower.receive().await?
                    };

                    println!("Received {} bytes from follower", b.len());

                    if let Ok(message) = String::from_utf8(b) {
                        queue.push_back(message.clone());

                        // DKG requires all-to-all communication.
                        if self.dkg {
                            for i2 in 0..followers.len() {
                                if i2 != i1 {
                                    let f2 = &mut followers[i2];
                                    f2.send(message.as_bytes().to_vec()).await?;
                                    println!("Sent {} bytes to follower", message.len());
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
