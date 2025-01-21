use bitvec::prelude::*;

use penumbra_sdk_proto::{penumbra::core::component::stake::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// Records information on a validator's uptime.
///
/// This structure provides two operations:
///
/// - recording that a validator did or did not sign a particular block;
/// - reporting how many of the last N blocks a validator has missed signing.
///
/// Internally, the `Uptime` uses a bit vector as a ring buffer, with a `1` bit
/// recording that the validator signed the block, and `0` recording that it
/// didn't.  For a new validator, the [`Uptime::new`] method initializes the bit
/// vector with all `1`s as a grace period to ensure that we don't need to
/// special-case genesis states, new validators, etc.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::Uptime", into = "pb::Uptime")]
pub struct Uptime {
    // Note: tracking this means we *could* in principle answer queries by
    // height, they just might be surprising for new validators (we just report
    // *failures* to sign, not didn't sign)
    //
    // can also possibly extend this impl to support resizing the window when we
    // get to that
    as_of_block_height: u64,
    signatures: BitVec<u8, Lsb0>,
}

impl std::fmt::Debug for Uptime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Uptime").finish()
    }
}

impl Uptime {
    /// Initialize an uptime tracker for a new validator.
    ///
    /// This method should not be used for existing validators.  Instead,
    /// deserialize the tracker (that should have been) created when the
    /// validator was created.
    pub fn new(initial_block_height: u64, signed_blocks_window_len: usize) -> Self {
        let signatures = bitvec![u8, Lsb0; 1; signed_blocks_window_len];

        Self {
            as_of_block_height: initial_block_height,
            signatures,
        }
    }

    /// Mark that the validator signed the block at the given height (`true`),
    /// or did not sign (`false`).
    ///
    /// This method errors only if the provided `height` isn't one greater than
    /// the current block height.  This should not happen in normal use (i.e.,
    /// it's probably reasonable to `expect` on the error), but the method
    /// takes an explicit height and checks it to flag misuse and detect bugs.
    pub fn mark_height_as_signed(&mut self, height: u64, signed: bool) -> anyhow::Result<()> {
        if height != self.as_of_block_height + 1 {
            anyhow::bail!(
                "Last block height was {} but next block height is {}",
                self.as_of_block_height,
                height
            );
        }

        // Use the bit vector as a ring buffer, overwriting the record for N blocks ago with this one.
        let index = (height as usize) % self.signatures.len();
        self.signatures.set(index, signed);
        self.as_of_block_height = height;

        Ok(())
    }

    /// Counts the number of missed blocks over the window.
    pub fn num_missed_blocks(&self) -> usize {
        self.signatures.iter_zeros().len()
    }

    /// Enumerates the missed blocks over the window in terms of absolute block height.
    pub fn missed_blocks(&self) -> impl Iterator<Item = u64> + DoubleEndedIterator + '_ {
        // The height of the latest block that's been recorded:
        let current_height = self.as_of_block_height;
        // The length of the window of blocks being recorded:
        let window_len = self.signatures.len();
        // The earliest height of a block that has been recorded:
        let earliest_height = current_height.saturating_sub(window_len as u64 - 1);
        // The range of block heights that have been recorded:
        let all_heights = earliest_height..=current_height;
        // Filter out the heights that were signed:
        all_heights.filter_map(move |height| {
            // Index the bit vector as the ring buffer that it is, and invert the bit corresponding
            // to this height to find out whether it was missed:
            let index = (height as usize) % window_len;
            let signed = self.signatures[index];
            Some(height).filter(|_| !signed)
        })
    }

    /// Returns the block height up to which this tracker has recorded.
    pub fn as_of_height(&self) -> u64 {
        self.as_of_block_height
    }

    /// Returns the size of the window of blocks being recorded.
    pub fn missed_blocks_window(&self) -> usize {
        self.signatures.len()
    }
}

impl DomainType for Uptime {
    type Proto = pb::Uptime;
}

impl From<Uptime> for pb::Uptime {
    fn from(mut val: Uptime) -> pb::Uptime {
        // canonicalize any unused data
        val.signatures.set_uninitialized(true);
        let window_len = val.signatures.len() as u32;
        let bitvec = val.signatures.into_vec();
        pb::Uptime {
            as_of_block_height: val.as_of_block_height,
            window_len,
            bitvec,
        }
    }
}

impl TryFrom<pb::Uptime> for Uptime {
    type Error = anyhow::Error;
    fn try_from(msg: pb::Uptime) -> Result<Uptime, Self::Error> {
        let mut signatures = BitVec::from_vec(msg.bitvec);
        if signatures.len() < msg.window_len as usize {
            anyhow::bail!("not enough data in bitvec buffer");
        }
        signatures.truncate(msg.window_len as usize);
        Ok(Uptime {
            signatures,
            as_of_block_height: msg.as_of_block_height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;
    use std::collections::VecDeque;

    #[test]
    fn counts_missed_blocks() {
        let window = 128;
        let mut uptime = Uptime::new(0, window);

        // Simulate missing every 4th block for a full window
        for h in 1..(window + 1) {
            uptime.mark_height_as_signed(h as u64, h % 4 != 0).unwrap();
        }
        assert_eq!(uptime.num_missed_blocks(), window / 4);

        // Now miss no blocks and check that the old data is forgotten
        for h in (window + 1)..(2 * window + 1) {
            uptime.mark_height_as_signed(h as u64, true).unwrap();
        }
        assert_eq!(uptime.num_missed_blocks(), 0);

        // Finally, check that the sanity-checking works
        assert!(uptime.mark_height_as_signed(0, true).is_err());
    }

    /// Basic check that if we miss block 1, we report that we missed block 1.
    #[test]
    fn enumerate_missed_first_block() {
        let window = 128;
        let mut uptime = Uptime::new(0, window);

        // Mark the first block as missed
        uptime.mark_height_as_signed(1, false).unwrap();
        let missed_blocks: Vec<_> = uptime.missed_blocks().collect();

        // Check that exactly the first block is missed
        assert_eq!(missed_blocks, vec![1]);
    }

    proptest! {
        /// Ensure that the `Uptime` struct simulates a fixed size queue of (height, signed) tuples,
        /// and that the `missed_blocks` iterator returns the correct missed blocks.
        #[test]
        fn enumerate_uptime_simulates_bounded_queue(
            (window_len, signed_blocks) in
                (1..=16usize).prop_flat_map(move |window_len| {
                    proptest::collection::vec(proptest::bool::ANY, 0..window_len * 2)
                        .prop_map(move |signed_blocks| (window_len, signed_blocks))
                })
        ) {
            // We're going to simulate the `Uptime` struct with a VecDeque of (height, signed)
            // tuples whose length we will keep bounded by the window length.
            let mut uptime = Uptime::new(0, window_len);
            let mut simulated_uptime = VecDeque::new();

            // For each (height, signed) tuple in our generated sequence, mark the height as signed
            // or not signed.
            for (height, signed) in signed_blocks.into_iter().enumerate() {
                // Convert the height to a u64 and add 1 because the `Uptime` struct starts out with
                // an internal height of 0:
                let height = height as u64 + 1;
                // Mark it using the real `Uptime` struct:
                uptime.mark_height_as_signed(height, signed).unwrap();
                // Mark it using our simulated `VecDeque`, taking care to keep its length bounded by
                // the window length:
                simulated_uptime.push_back((height, signed));
                if simulated_uptime.len() > window_len {
                    simulated_uptime.pop_front();
                }
            }

            // Compare the missed blocks from the real `Uptime` struct with the simulated `VecDeque`:
            let missed_blocks: Vec<_> = uptime.missed_blocks().collect();

            // Retain only the heights from the simulated `VecDeque` that were not signed:
            simulated_uptime.retain(|(_, signed)| !signed);
            let simulated_missed_blocks: Vec<_> =
                simulated_uptime.into_iter().map(|(height, _)| height).collect();

            prop_assert_eq!(missed_blocks, simulated_missed_blocks);
        }
    }

    #[test]
    fn proto_round_trip() {
        // make a weird size window
        let mut uptime = Uptime::new(0, 113);
        // Fill it with some data
        for h in 1..300u64 {
            uptime.mark_height_as_signed(h, h % 13 != 0).unwrap();
        }

        let bytes = uptime.encode_to_vec();
        let uptime2 = Uptime::decode(bytes.as_slice()).unwrap();
        assert_eq!(uptime, uptime2);
    }
}
