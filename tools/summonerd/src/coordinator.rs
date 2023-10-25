use std::time::Duration;

use anyhow::Result;

use crate::{participant::Participant, phase::Phase, queue::ParticipantQueue, storage::Storage};

const QUEUE_SLEEP_TIME_SECS: u64 = 1;

pub struct Coordinator {
    storage: Storage,
    queue: ParticipantQueue,
}

impl Coordinator {
    pub fn new(storage: Storage, queue: ParticipantQueue) -> Self {
        Self { storage, queue }
    }

    pub async fn run<P: Phase + 'static>(mut self) -> Result<()> {
        loop {
            let participant_count = self.queue.len().await;
            tracing::debug!(
                participant_count = participant_count,
                "top of coordinator loop"
            );
            // 1. Inform all participants of their position
            self.queue.inform_all().await?;
            // 2. Select a contributor (we may have to sleep repeatedly till the queue gets at
            //    least one member).
            let (contributor, _) = loop {
                if let Some(out) = self.queue.prune_and_pop().await {
                    break out;
                }
                tokio::time::sleep(Duration::from_secs(QUEUE_SLEEP_TIME_SECS)).await;
            };
            // 3. Get their contribution, or time out.
            self.contribute::<P>(contributor).await?;
        }
    }

    #[tracing::instrument(skip_all, fields(address = ?contributor.address()))]
    async fn contribute<P: Phase>(&mut self, contributor: Participant) -> Result<()> {
        let address = contributor.address();
        match tokio::time::timeout(
            Duration::from_secs(P::CONTRIBUTION_TIME_SECS),
            self.contribute_inner::<P>(contributor),
        )
        .await
        {
            Ok(Ok(_)) => Ok(()),
            Err(_) => {
                tracing::info!("STRIKE (timeout)");
                self.storage.strike(&address).await?;
                Ok(())
            }
            Ok(Err(e)) => Err(e),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn contribute_inner<P: Phase>(&mut self, mut contributor: Participant) -> Result<()> {
        let address = contributor.address();
        let parent = P::current_crs(&self.storage)
            .await?
            .expect("the phase should've been initialized by now");
        let maybe = contributor.contribute::<P>(&parent).await?;
        if let Some(unvalidated) = maybe {
            tracing::debug!("validating contribution");
            let root = P::fetch_root(&self.storage).await?;
            let maybe_contribution = tokio::task::spawn_blocking(move || {
                if let Some(contribution) = P::validate(&root, unvalidated) {
                    if P::is_linked_to(&contribution, &parent) {
                        return Some(contribution);
                    }
                }
                None
            })
            .await?;
            if let Some(contribution) = maybe_contribution {
                P::commit_contribution(&self.storage, address, contribution).await?;
                contributor
                    .confirm(self.storage.current_slot(P::MARKER).await?)
                    .await?;
                return Ok(());
            }
        }
        tracing::info!("STRIKE (invalid or partial contribution)");
        self.storage.strike(&address).await?;
        return Ok(());
    }
}
