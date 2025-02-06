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

            const EPOCH_LEN: usize = 20;
            const ASSET_LEN: usize = 32;
            const POWER_LEN: usize = 8;
            const PART0: &'static str = "funding/lqt/v1/votes/";

            fn format_epoch(epoch_index: u64) -> String {
                format!("{epoch_index:0w$}", w = EPOCH_LEN)
            }

            pub mod total {
                use super::*;

                const PART1: &'static str = "/total/";

                const KEY_LEN: usize = PART0.len() + EPOCH_LEN + PART1.len();

                pub(crate) fn key(epoch_index: u64) -> Vec<u8> {
                    let mut bytes = Vec::with_capacity(KEY_LEN);

                    bytes.extend_from_slice(PART0.as_bytes());
                    bytes.extend_from_slice(format_epoch(epoch_index).as_bytes());
                    bytes.extend_from_slice(PART1.as_bytes());

                    bytes
                }
            }

            pub mod by_asset {
                use super::*;

                const PART1: &'static str = "/by_asset/";

                pub mod total {
                    use super::*;

                    const PART2: &'static str = "total/";
                    const KEY_LEN: usize =
                        PART0.len() + EPOCH_LEN + PART1.len() + PART2.len() + ASSET_LEN;

                    pub(crate) fn key(epoch_index: u64, asset: asset::Id) -> Vec<u8> {
                        let mut bytes = Vec::with_capacity(KEY_LEN);

                        bytes.extend_from_slice(PART0.as_bytes());
                        bytes.extend_from_slice(format_epoch(epoch_index).as_bytes());
                        bytes.extend_from_slice(PART1.as_bytes());
                        bytes.extend_from_slice(PART2.as_bytes());
                        bytes.extend_from_slice(asset.to_bytes().as_slice());

                        bytes
                    }
                }

                pub mod ranked {
                    use super::*;

                    const PART2: &'static str = "ranked/";
                    const EPOCH_PREFIX_LEN: usize =
                        PART0.len() + EPOCH_LEN + PART1.len() + PART2.len();

                    pub(crate) fn prefix(epoch_index: u64) -> [u8; EPOCH_PREFIX_LEN] {
                        let mut bytes = [0u8; EPOCH_PREFIX_LEN];

                        let rest = &mut bytes;
                        let (bytes_part0, rest) = rest.split_at_mut(PART0.len());
                        let (bytes_epoch, rest) = rest.split_at_mut(EPOCH_LEN);
                        let (bytes_part1, rest) = rest.split_at_mut(PART1.len());
                        let (bytes_part2, _) = rest.split_at_mut(PART2.len());

                        bytes_part0.copy_from_slice(PART0.as_bytes());
                        bytes_epoch.copy_from_slice(format_epoch(epoch_index).as_bytes());
                        bytes_part1.copy_from_slice(PART1.as_bytes());
                        bytes_part2.copy_from_slice(PART2.as_bytes());

                        bytes
                    }

                    const KEY_LEN: usize = EPOCH_PREFIX_LEN + POWER_LEN + ASSET_LEN;

                    pub(crate) fn key(epoch_index: u64, power: u64, asset: asset::Id) -> Vec<u8> {
                        let mut bytes = Vec::with_capacity(KEY_LEN);

                        bytes.extend_from_slice(&prefix(epoch_index));
                        bytes.extend_from_slice(&((!power).to_be_bytes()));
                        bytes.extend_from_slice(&asset.to_bytes());

                        bytes
                    }

                    pub(crate) fn parse_key(key: &[u8]) -> anyhow::Result<(u64, asset::Id)> {
                        anyhow::ensure!(
                            key.len() == KEY_LEN,
                            "key length was {}, expected {}",
                            key.len(),
                            KEY_LEN
                        );
                        let rest = key;
                        let (_bytes_prefix, rest) = rest.split_at(EPOCH_PREFIX_LEN);
                        let (bytes_power, rest) = rest.split_at(POWER_LEN);
                        let (bytes_asset, _) = rest.split_at(ASSET_LEN);

                        let power = !u64::from_be_bytes(bytes_power.try_into()?);
                        let asset = asset::Id::try_from(bytes_asset)?;

                        Ok((power, asset))
                    }
                }
            }

            pub mod by_voter {

                use penumbra_sdk_keys::{address::ADDRESS_LEN_BYTES, Address};
                use penumbra_sdk_stake::IDENTITY_KEY_LEN_BYTES;

                use super::*;

                const PART1: &'static str = "/by_voter/";

                pub mod total {

                    use penumbra_sdk_stake::IdentityKey;

                    use super::*;

                    const PART2: &'static str = "total/";
                    const KEY_LEN: usize = PART0.len()
                        + EPOCH_LEN
                        + PART1.len()
                        + PART2.len()
                        + ASSET_LEN
                        + ADDRESS_LEN_BYTES
                        + IDENTITY_KEY_LEN_BYTES;

                    pub(crate) fn key(
                        epoch_index: u64,
                        asset: asset::Id,
                        addr: &Address,
                        validator: &IdentityKey,
                    ) -> Vec<u8> {
                        let mut bytes = Vec::with_capacity(KEY_LEN);

                        bytes.extend_from_slice(PART0.as_bytes());
                        bytes.extend_from_slice(format_epoch(epoch_index).as_bytes());
                        bytes.extend_from_slice(PART1.as_bytes());
                        bytes.extend_from_slice(PART2.as_bytes());
                        bytes.extend_from_slice(asset.to_bytes().as_slice());
                        bytes.extend_from_slice(addr.to_vec().as_slice());
                        bytes.extend_from_slice(validator.to_bytes().as_slice());

                        bytes
                    }
                }

                pub mod ranked {
                    use penumbra_sdk_stake::IdentityKey;

                    use super::*;

                    const PART2: &'static str = "ranked/";
                    const EPOCH_ASSET_PREFIX_LEN: usize =
                        PART0.len() + EPOCH_LEN + PART1.len() + PART2.len() + ASSET_LEN;

                    pub(crate) fn prefix(
                        epoch_index: u64,
                        asset: asset::Id,
                    ) -> [u8; EPOCH_ASSET_PREFIX_LEN] {
                        let mut bytes = [0u8; EPOCH_ASSET_PREFIX_LEN];

                        let rest = &mut bytes;
                        let (bytes_part0, rest) = rest.split_at_mut(PART0.len());
                        let (bytes_epoch, rest) = rest.split_at_mut(EPOCH_LEN);
                        let (bytes_part1, rest) = rest.split_at_mut(PART1.len());
                        let (bytes_part2, rest) = rest.split_at_mut(PART2.len());
                        let (bytes_asset, _) = rest.split_at_mut(ASSET_LEN);

                        bytes_part0.copy_from_slice(PART0.as_bytes());
                        bytes_epoch.copy_from_slice(format_epoch(epoch_index).as_bytes());
                        bytes_part1.copy_from_slice(PART1.as_bytes());
                        bytes_part2.copy_from_slice(PART2.as_bytes());
                        bytes_asset.copy_from_slice(&asset.to_bytes());

                        bytes
                    }

                    const KEY_LEN: usize = EPOCH_ASSET_PREFIX_LEN
                        + POWER_LEN
                        + ADDRESS_LEN_BYTES
                        + IDENTITY_KEY_LEN_BYTES;

                    pub(crate) fn key(
                        epoch_index: u64,
                        asset: asset::Id,
                        power: u64,
                        addr: &Address,
                        validator: &IdentityKey,
                    ) -> Vec<u8> {
                        let mut bytes = Vec::with_capacity(KEY_LEN);

                        bytes.extend_from_slice(&prefix(epoch_index, asset));
                        bytes.extend_from_slice(&((!power).to_be_bytes()));
                        bytes.extend_from_slice(&addr.to_vec());
                        bytes.extend_from_slice(&validator.to_bytes());

                        bytes
                    }

                    pub(crate) fn parse_key(
                        key: &[u8],
                    ) -> anyhow::Result<(u64, Address, IdentityKey)> {
                        anyhow::ensure!(
                            key.len() == KEY_LEN,
                            "key length was {}, expected {}",
                            key.len(),
                            KEY_LEN
                        );
                        let rest = key;
                        let (_bytes_prefix, rest) = rest.split_at(EPOCH_ASSET_PREFIX_LEN);
                        let (bytes_power, rest) = rest.split_at(POWER_LEN);
                        let (bytes_addr, rest) = rest.split_at(ADDRESS_LEN_BYTES);
                        let (bytes_ik, _) = rest.split_at(IDENTITY_KEY_LEN_BYTES);

                        let power = !u64::from_be_bytes(bytes_power.try_into()?);
                        let addr = Address::try_from(bytes_addr)?;
                        let ik = IdentityKey(bytes_ik.try_into()?);

                        Ok((power, addr, ik))
                    }
                }
            }
        }
    }
}
