use std::{collections::HashMap, time::Duration};

use anyhow::{anyhow, Context, Result};
use penumbra_keys::Address;
use penumbra_proto::tools::summoning::v1alpha1::CeremonyCrs;
use rand::{rngs::OsRng, seq::SliceRandom};
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::{participant::Participant, storage::Storage};

pub struct Coordinator {
    storage: Storage,
    participants: HashMap<Address, Participant>,
    new_participant_rx: mpsc::Receiver<Participant>,
}

impl Coordinator {
    pub fn new(storage: Storage) -> (Self, mpsc::Sender<Participant>) {
        let (new_participant_tx, new_participant_rx) = mpsc::channel(9001);
        (
            Self {
                storage,
                participants: HashMap::new(),
                new_participant_rx,
            },
            new_participant_tx,
        )
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            // 0. Wait for the first participant
            if self.participants.is_empty() {
                self.wait_for_participant().await?;
            }
            // 1. Check for new connections, but don't wait for them.
            self.dequeue_participants()?;
            // 2. Score connections
            self.prune_participants();
            let ranked = self.score_participants();
            // 3. Update everyone on status.
            self.inform_participants_of_status(&ranked).await;
            // In theory ranked could've become empty for some reason in the meantime
            if ranked.is_empty() {
                continue;
            }
            // 5. Get contribution, or error if they don't respond quickly enough
            let contributor = ranked[0];
            self.contribute(contributor).await?;
            // 6. Remove from pool regardless of what happened
            self.participants.remove(&contributor);
        }
    }
}

impl Coordinator {
    async fn wait_for_participant(&mut self) -> Result<()> {
        if let Some(participant) = self.new_participant_rx.recv().await {
            let address = participant.address();
            tracing::info!(?address, "has been added as a participant");
            self.participants.insert(address, participant);
            Ok(())
        } else {
            Err(anyhow!("Participant queue was closed"))
        }
    }

    fn dequeue_participants(&mut self) -> Result<()> {
        loop {
            match self.new_participant_rx.try_recv() {
                Ok(participant) => {
                    let address = participant.address();
                    tracing::info!(?address, "has been added as a participant");
                    self.participants.insert(address, participant);
                }
                Err(TryRecvError::Empty) => return Ok(()),
                Err(e @ TryRecvError::Disconnected) => {
                    return Err(e).with_context(|| "Channel with incoming connections was closed")
                }
            }
        }
    }

    fn prune_participants(&mut self) {
        self.participants
            .retain(|_, connection| connection.is_live());
    }

    fn score_participants(&self) -> Vec<Address> {
        let mut out: Vec<Address> = self.participants.keys().cloned().collect();
        out.shuffle(&mut OsRng);
        out
    }

    async fn inform_participants_of_status(&mut self, ranked: &[Address]) {
        for (i, address) in ranked.iter().enumerate() {
            let connection = self
                .participants
                .get(address)
                .expect("Ranked participants are chosen from the set of connections");
            if let Err(e) = connection.try_notify(i as u32, ranked.len() as u32) {
                tracing::info!(?e, "pruning connection that we failed to notify");
                self.participants.remove(address);
            };
        }
    }

    #[tracing::instrument(skip(self))]
    async fn contribute(&mut self, contributor: Address) -> Result<()> {
        match tokio::time::timeout(
            // 10 minutes
            Duration::from_secs(10 * 60),
            self.contribute_inner(contributor),
        )
        .await
        {
            Ok(Ok(_)) => Ok(()),
            Err(_) => {
                tracing::info!("timeout when asking for contribution");
                Ok(())
            }
            Ok(Err(e)) => Err(e),
        }
    }

    async fn contribute_inner(&mut self, contributor: Address) -> Result<()> {
        // TODO: Verify contribution
        let parent = self.storage.current_crs().await?;
        let new_crs = match self
            .participants
            .get_mut(&contributor)
            .expect("We ask for the contributions of participants we're connected to")
            .contribute(parent)
            .await
        {
            Ok(crs) => crs,
            Err(e) => {
                // TODO: Maybe remove permanently
                tracing::info!(?e, "Made a bad contribution");
                return Ok(());
            }
        };
        // TODO: Run actual validation math
        self.storage
            .commit_contribution(contributor, &new_crs)
            .await?;
        Ok(())
    }
}
