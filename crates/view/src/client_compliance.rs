//! Compliance extensions for ViewClient.
//!
//! This module provides compliance-related methods that wrap ViewClient calls
//! and provide convenient access to compliance registry state.

use anyhow::Result;
use futures::FutureExt;
use penumbra_sdk_asset::asset;
use penumbra_sdk_compliance::ComplianceLeaf;
use penumbra_sdk_keys::{keys::AddressComplianceKey, Address};
use penumbra_sdk_tct::StateCommitment;
use std::{future::Future, pin::Pin};

use crate::ViewClient;

/// Compliance extensions for ViewClient.
///
/// These methods provide convenient access to compliance registry state.
pub trait ViewClientComplianceExt: ViewClient {
    /// Check if an asset is regulated (requires compliance).
    ///
    /// # Implementation
    /// Queries the compliance registry (via ViewClient::compliance_asset_status)
    /// to check if the asset is regulated.
    ///
    /// Returns `true` if the asset is registered and regulated, `false` otherwise.
    fn is_asset_regulated(
        &mut self,
        asset_id: asset::Id,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'static>> {
        let status_future = self.compliance_asset_status(asset_id);
        async move {
            let status = status_future.await?;
            // Return true only if the asset is explicitly regulated
            Ok(status.unwrap_or(false))
        }
        .boxed()
    }

    /// Get the compliance leaf for a specific address and asset.
    ///
    /// Fetches the registered ComplianceLeaf from the chain. This ensures
    /// the leaf used in proofs matches what was actually registered on-chain.
    ///
    /// # Returns
    /// The ComplianceLeaf if the user is registered, or an error if not registered.
    fn get_compliance_leaf(
        &mut self,
        address: Address,
        asset_id: asset::Id,
    ) -> Pin<Box<dyn Future<Output = Result<ComplianceLeaf>> + Send + 'static>> {
        let leaf_future = self.compliance_user_leaf(address.clone(), asset_id);
        async move {
            let response = leaf_future.await?;

            if !response.is_registered {
                anyhow::bail!("User not registered for this asset in compliance registry");
            }

            let proto_leaf = response
                .leaf
                .ok_or_else(|| anyhow::anyhow!("User registered but leaf missing from response"))?;

            // Parse the proto leaf into native ComplianceLeaf
            let address: Address = proto_leaf
                .address
                .ok_or_else(|| anyhow::anyhow!("missing address in leaf"))?
                .try_into()?;

            let key_proto = proto_leaf
                .key
                .ok_or_else(|| anyhow::anyhow!("missing key in leaf"))?;
            let key_bytes: [u8; 32] = key_proto
                .inner
                .as_slice()
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid key length"))?;
            let key_element = decaf377::Encoding(key_bytes)
                .vartime_decompress()
                .map_err(|_| anyhow::anyhow!("invalid key encoding"))?;
            let key = AddressComplianceKey::new(key_element);

            let asset_id: penumbra_sdk_asset::asset::Id = proto_leaf
                .asset_id
                .ok_or_else(|| anyhow::anyhow!("missing asset_id in leaf"))?
                .try_into()?;

            Ok(ComplianceLeaf {
                address,
                key,
                asset_id,
            })
        }
        .boxed()
    }

    /// Get the compliance tree anchors from the chain.
    ///
    /// Returns (compliance_anchor, asset_anchor) - the roots of the user tree
    /// and asset tree respectively.
    fn get_compliance_anchors(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(StateCommitment, StateCommitment)>> + Send + 'static>>
    {
        let anchors_future = self.compliance_anchors();
        async move {
            let (compliance_anchor, asset_anchor) = anchors_future.await?;
            Ok((compliance_anchor, asset_anchor))
        }
        .boxed()
    }

    /// Get the Merkle proofs needed for compliance ZK proofs.
    ///
    /// This method queries the chain for:
    /// - User's Merkle path and position in the compliance tree
    /// - Asset's Merkle path and position in the asset tree
    /// - Both tree anchors (roots)
    ///
    /// Returns a `ComplianceMerkleProofsData` with all the data needed for plans.
    fn get_compliance_merkle_proofs(
        &mut self,
        wallet_id: Address,
        asset_id: asset::Id,
    ) -> Pin<Box<dyn Future<Output = Result<ComplianceMerkleProofsData>> + Send + 'static>> {
        let proofs_future = self.compliance_merkle_proofs(wallet_id, asset_id);
        async move {
            let response = proofs_future.await?;
            ComplianceMerkleProofsData::try_from_proto(response)
        }
        .boxed()
    }
}

/// Data structure containing parsed Merkle proofs for compliance.
/// This is the Rust-native equivalent of ComplianceMerkleProofsResponse.
#[derive(Debug, Clone)]
pub struct ComplianceMerkleProofsData {
    pub user_registered: bool,
    pub asset_registered: bool,
    pub is_regulated: bool,
    pub compliance_path: penumbra_sdk_compliance::structs::MerklePath,
    pub compliance_position: u64,
    pub asset_path: penumbra_sdk_compliance::structs::MerklePath,
    pub asset_position: u64,
    pub compliance_anchor: StateCommitment,
    pub asset_anchor: StateCommitment,
}

impl ComplianceMerkleProofsData {
    /// Convert from the proto response to native types.
    pub fn try_from_proto(
        response: penumbra_sdk_proto::view::v1::ComplianceMerkleProofsResponse,
    ) -> Result<Self> {
        use decaf377::Fq;
        use penumbra_sdk_compliance::structs::MerklePathLayer;

        // Parse compliance path - siblings are stored as raw bytes
        let compliance_path = if let Some(path) = response.compliance_path {
            penumbra_sdk_compliance::structs::MerklePath {
                layers: path
                    .layers
                    .into_iter()
                    .map(|layer| MerklePathLayer {
                        siblings: layer.siblings,
                    })
                    .collect(),
            }
        } else {
            penumbra_sdk_compliance::structs::MerklePath { layers: vec![] }
        };

        // Parse asset path - siblings are stored as raw bytes
        let asset_path = if let Some(path) = response.asset_path {
            penumbra_sdk_compliance::structs::MerklePath {
                layers: path
                    .layers
                    .into_iter()
                    .map(|layer| MerklePathLayer {
                        siblings: layer.siblings,
                    })
                    .collect(),
            }
        } else {
            penumbra_sdk_compliance::structs::MerklePath { layers: vec![] }
        };

        // Parse anchors
        let compliance_anchor_bytes: [u8; 32] = response
            .compliance_anchor
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid compliance_anchor length"))?;
        let compliance_anchor = StateCommitment(
            Fq::from_bytes_checked(&compliance_anchor_bytes)
                .map_err(|_| anyhow::anyhow!("invalid compliance_anchor Fq"))?,
        );

        let asset_anchor_bytes: [u8; 32] = response
            .asset_anchor
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid asset_anchor length"))?;
        let asset_anchor = StateCommitment(
            Fq::from_bytes_checked(&asset_anchor_bytes)
                .map_err(|_| anyhow::anyhow!("invalid asset_anchor Fq"))?,
        );

        Ok(Self {
            user_registered: response.user_registered,
            asset_registered: response.asset_registered,
            is_regulated: response.is_regulated,
            compliance_path,
            compliance_position: response.compliance_position,
            asset_path,
            asset_position: response.asset_position,
            compliance_anchor,
            asset_anchor,
        })
    }
}

// Blanket implementation for all ViewClient implementors
impl<T: ViewClient + ?Sized> ViewClientComplianceExt for T {}
