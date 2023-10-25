use anyhow::Result;
use std::{collections::HashSet, sync::Arc};

use penumbra_keys::Address;
use penumbra_num::Amount;
use tokio::sync::RwLock;

use crate::participant::Participant;

/// The inner struct for "normal" manipulation outside of the concurrent data structure
struct Inner {
    // Invariant: this is always sorted by the amount.
    // Invariant: no two participants have the same address
    sorted: Vec<(Participant, Amount)>,
    // Used to keep track of all the addresses in the sorted vec.
    addresses: HashSet<Address>,
}

impl Inner {
    fn new() -> Self {
        Inner {
            sorted: Vec::new(),
            addresses: HashSet::new(),
        }
    }

    /// Get the number of participants in the queue
    fn len(&self) -> usize {
        self.sorted.len()
    }

    /// Insert a participant and their bid into the queue.
    ///
    /// If the participant is already in the queue (as determined by having the same address),
    /// then this may push them higher in the queue if the bid is higher.
    ///
    /// Their bid being lower should never happen in practice.
    ///
    /// We also don't expect the case of duplicate participants to happen either, but we should
    /// handle it.
    fn push(&mut self, participant: Participant, bid: Amount) {
        let address = participant.address();
        // If the participant is already present in the queue, remove them.
        if self.addresses.contains(&address) {
            // Note, we don't remove them from the address list, because we'll be inserting soon.
            self.sorted.retain(|(p, _)| p.address() != address);
        }
        let position = match self.sorted.binary_search_by(|(_, a)| a.cmp(&bid)) {
            // If we find a participant with the same bid, insert before them.
            Ok(pos) => pos,
            // If no participant with the bid is found, this is the correct position.
            Err(pos) => pos,
        };
        self.sorted.insert(position, (participant, bid));
        self.addresses.insert(address);
    }

    /// Remove all participants that aren't live
    fn prune(&mut self) {
        self.sorted.retain(|(participant, _)| {
            let live = participant.is_live();
            // Make sure to remove from the addresses list as well.
            if !live {
                self.addresses.remove(&participant.address());
            }
            live
        })
    }

    /// Remove the participant with the highest bid from this queue.
    ///
    /// This will return None if the queue is empty.
    fn pop(&mut self) -> Option<(Participant, Amount)> {
        let result = self.sorted.pop();
        if let Some((ref participant, _)) = result {
            self.addresses.remove(&participant.address());
        }
        result
    }

    /// Return the highest bid in the queue, if not empty.
    ///
    /// This will return None if the queue is empty.
    fn top_bid(&self) -> Option<Amount> {
        // The last item in the sorted vec will have the highest bid, use that.
        self.sorted.last().map(|(_, amount)| *amount)
    }

    /// Return participants, and their bids, in ascending order of bids.
    fn iter(&self) -> impl Iterator<Item = &(Participant, Amount)> {
        self.sorted.iter()
    }
}

/// A thread safe queue of participants, sorted by bid amounts.
#[derive(Clone)]
pub struct ParticipantQueue {
    participants: Arc<RwLock<Inner>>,
}

impl ParticipantQueue {
    pub fn new() -> Self {
        Self {
            participants: Arc::new(RwLock::new(Inner::new())),
        }
    }

    /// Get the length of the queue
    pub async fn len(&self) -> usize {
        self.participants.read().await.len()
    }

    /// Push a new participant into the queue, given their bid.
    pub async fn push(&self, participant: Participant, bid: Amount) {
        self.participants.write().await.push(participant, bid);
    }

    /// Remove inactive connections, and then remove out the highest bidder.
    pub async fn prune_and_pop(&self) -> Option<(Participant, Amount)> {
        let mut participants = self.participants.write().await;
        participants.prune();
        participants.pop()
    }

    #[tracing::instrument(skip(self))]
    async fn inform(&self, filter: Option<Address>) -> Result<()> {
        let participants = self.participants.read().await;
        let top_bid = match participants.top_bid() {
            None => return Ok(()),
            Some(bid) => bid,
        };
        let participant_count = participants.len();
        for (i, (participant, bid)) in participants.iter().enumerate() {
            let address = participant.address();
            match filter {
                Some(f) if f != address => continue,
                _ => {}
            }
            // Ignore failures (besides logging), let pruning happen later.
            if let Err(e) =
                participant.try_notify(i as u32, participant_count as u32, top_bid, *bid)
            {
                tracing::debug!(?address, ?e, "failed to notify");
            }
        }
        Ok(())
    }

    /// Inform one participant of their position in the queue.
    pub async fn inform_one(&self, address: Address) -> Result<()> {
        self.inform(Some(address)).await
    }

    /// Informal all participants of their position in the queue.
    pub async fn inform_all(&self) -> Result<()> {
        self.inform(None).await
    }
}
