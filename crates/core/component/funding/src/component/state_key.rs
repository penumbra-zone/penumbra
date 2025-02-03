pub fn funding_parameters() -> &'static str {
    "funding/parameters"
}

pub mod lqt {
    pub mod v1 {
        pub mod nullifier {
            use penumbra_sdk_sct::Nullifier;

            /// A nullifier set indexed by epoch, mapping each epoch to its corresponding `TransactionId`.
            pub(crate) fn key(epoch_index: u64, nullifier: &Nullifier) -> String {
                format!("funding/lqt/v1/nullifier/{epoch_index:020}/lookup/{nullifier}")
            }
        }

        pub mod votes {
            use penumbra_sdk_asset::asset;
            use penumbra_sdk_keys::{address::ADDRESS_LEN_BYTES, Address};

            const PART0: &'static str = "funding/lqt/v1/votes/";
            const EPOCH_LEN: usize = 20;
            const PART1: &'static str = "/by_asset/";
            const PREFIX_LEN: usize = PART0.len() + EPOCH_LEN + PART1.len();

            /// A prefix for accessing the votes in a given epoch, c.f. [`power_asset_address`];
            pub(crate) fn prefix(epoch_index: u64) -> [u8; PREFIX_LEN] {
                let mut bytes = [0u8; PREFIX_LEN];

                let rest = &mut bytes;
                let (bytes_part0, rest) = rest.split_at_mut(PART0.len());
                let (bytes_epoch_index, bytes_part1) = rest.split_at_mut(EPOCH_LEN);

                bytes_part0.copy_from_slice(PART0.as_bytes());
                bytes_epoch_index
                    .copy_from_slice(format!("{epoch_index:0w$}", w = EPOCH_LEN).as_bytes());
                bytes_part1.copy_from_slice(PART1.as_bytes());

                bytes
            }

            const ASSET_LEN: usize = 32;
            const POWER_LEN: usize = 8;
            const RECEIPT_LEN: usize = PREFIX_LEN + ASSET_LEN + POWER_LEN + ADDRESS_LEN_BYTES;

            /// When present, indicates that an address voted for a particular asset, with a given power.
            ///
            /// To get the values ordered by descending voting power, use [`prefix`];
            pub(crate) fn receipt(
                epoch_index: u64,
                asset: asset::Id,
                power: u64,
                voter: &Address,
            ) -> [u8; RECEIPT_LEN] {
                let mut bytes = [0u8; RECEIPT_LEN];

                let rest = &mut bytes;
                let (bytes_prefix, rest) = rest.split_at_mut(PREFIX_LEN);
                let (bytes_asset, rest) = rest.split_at_mut(ASSET_LEN);
                let (bytes_power, bytes_voter) = rest.split_at_mut(POWER_LEN);

                bytes_prefix.copy_from_slice(&prefix(epoch_index));
                bytes_asset.copy_from_slice(&asset.to_bytes());
                bytes_power.copy_from_slice(&((!power).to_be_bytes()));
                bytes_voter.copy_from_slice(&voter.to_vec());

                bytes
            }

            /// Parse the output of [`receipt`] back into its parts.
            ///
            /// We return a `Vec<u8>` instead of an `Address`, because tallying
            /// wants to defer parsing the address until later, comparing encodings instead.
            pub(crate) fn parse_receipt(key: &[u8]) -> anyhow::Result<(asset::Id, u64, Vec<u8>)> {
                anyhow::ensure!(
                    key.len() == RECEIPT_LEN,
                    "key length was {}, expected {}",
                    key.len(),
                    RECEIPT_LEN
                );
                let rest = key;
                let (_bytes_prefix, rest) = rest.split_at(PREFIX_LEN);
                let (bytes_asset, rest) = rest.split_at(ASSET_LEN);
                let (bytes_power, bytes_voter) = rest.split_at(POWER_LEN);

                let asset = asset::Id::try_from(bytes_asset)?;
                let power = u64::from_be_bytes(bytes_asset.try_into()?);
                let voter = bytes_voter.to_vec();

                Ok((asset, power, voter))
            }
        }
    }
}
