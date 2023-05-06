use bitvec::prelude::*;

use penumbra_proto::{core::stake::v1alpha1 as pb, DomainType, TypeUrl};
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
    pub fn mark_height_as_signed(
        &mut self,
        height: u64,
        signed: bool,
    ) -> Result<(), anyhow::Error> {
        if height != self.as_of_block_height + 1 {
            return Err(anyhow::anyhow!(
                "Last block height was {} but next block height is {}",
                self.as_of_block_height,
                height
            ));
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
}

impl TypeUrl for Uptime {
    const TYPE_URL: &'static str = "/penumbra.core.stake.v1alpha1.Uptime";
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
            return Err(anyhow::anyhow!("not enough data in bitvec buffer"));
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
