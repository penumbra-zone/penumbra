use cnidarium::Storage;
use penumbra_sdk_asset::asset;
use penumbra_sdk_keys::Address;
use penumbra_sdk_proto::core::component::compliance::v1::{
    query_service_server::QueryService, ComplianceAnchorsRequest, ComplianceAnchorsResponse,
    ComplianceAssetStatusRequest, ComplianceAssetStatusResponse, ComplianceMerkleProofsRequest,
    ComplianceMerkleProofsResponse, ComplianceUserLeafRequest, ComplianceUserLeafResponse,
    MerklePath, MerklePathLayer,
};
use tonic::Status;
use tracing::instrument;

use crate::registry::ComplianceRegistryRead;

/// gRPC server for compliance registry queries.
pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl QueryService for Server {
    #[instrument(skip(self, request))]
    async fn compliance_asset_status(
        &self,
        request: tonic::Request<ComplianceAssetStatusRequest>,
    ) -> Result<tonic::Response<ComplianceAssetStatusResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let request = request.into_inner();
        let asset_id: asset::Id = request
            .asset_id
            .ok_or_else(|| Status::invalid_argument("missing asset_id"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("could not parse asset_id: {e}")))?;

        // Query the compliance registry for the asset's regulation status
        let status = state
            .get_asset_status(asset_id)
            .await
            .map_err(|e| Status::internal(format!("failed to query asset status: {e}")))?;

        let response = match status {
            Some(is_regulated) => {
                tracing::debug!(?asset_id, is_regulated, "found asset regulation status");
                ComplianceAssetStatusResponse {
                    asset_id: Some(asset_id.into()),
                    is_registered: true,
                    is_regulated,
                }
            }
            None => {
                tracing::debug!(?asset_id, "asset not registered in compliance system");
                ComplianceAssetStatusResponse {
                    asset_id: Some(asset_id.into()),
                    is_registered: false,
                    is_regulated: false,
                }
            }
        };

        Ok(tonic::Response::new(response))
    }

    #[instrument(skip(self, _request))]
    async fn compliance_anchors(
        &self,
        _request: tonic::Request<ComplianceAnchorsRequest>,
    ) -> Result<tonic::Response<ComplianceAnchorsResponse>, Status> {
        let state = self.storage.latest_snapshot();

        // Get the user tree root (compliance_anchor)
        let user_tree_root = state
            .get_user_tree_root()
            .await
            .map_err(|e| Status::internal(format!("failed to get user tree root: {e}")))?;

        // Get the asset tree root (asset_anchor)
        let asset_tree_root = state
            .get_asset_tree_root()
            .await
            .map_err(|e| Status::internal(format!("failed to get asset tree root: {e}")))?;

        tracing::debug!(
            ?user_tree_root,
            ?asset_tree_root,
            "returning compliance anchors"
        );

        let response = ComplianceAnchorsResponse {
            user_tree_root: user_tree_root.0.to_bytes().to_vec(),
            asset_tree_root: asset_tree_root.0.to_bytes().to_vec(),
        };

        Ok(tonic::Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn compliance_merkle_proofs(
        &self,
        request: tonic::Request<ComplianceMerkleProofsRequest>,
    ) -> Result<tonic::Response<ComplianceMerkleProofsResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let request = request.into_inner();

        // Parse address (Address)
        let address: Address = request
            .address
            .ok_or_else(|| Status::invalid_argument("missing address"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("could not parse address: {e}")))?;

        // Parse asset_id
        let asset_id: asset::Id = request
            .asset_id
            .ok_or_else(|| Status::invalid_argument("missing asset_id"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("could not parse asset_id: {e}")))?;

        // Get the tree roots (anchors)
        let compliance_anchor = state
            .get_user_tree_root()
            .await
            .map_err(|e| Status::internal(format!("failed to get user tree root: {e}")))?;

        let asset_anchor = state
            .get_asset_tree_root()
            .await
            .map_err(|e| Status::internal(format!("failed to get asset tree root: {e}")))?;

        // Look up user's position in compliance tree
        let user_position = state
            .get_user_leaf_position(&address, asset_id)
            .await
            .map_err(|e| Status::internal(format!("failed to look up user position: {e}")))?;

        // Look up asset's position in asset tree
        let asset_position_opt = state
            .get_asset_index(asset_id)
            .await
            .map_err(|e| Status::internal(format!("failed to look up asset position: {e}")))?;

        // Look up asset regulation status
        let asset_status = state
            .get_asset_status(asset_id)
            .await
            .map_err(|e| Status::internal(format!("failed to query asset status: {e}")))?;

        // Build the response based on what was found
        let (user_registered, compliance_path, compliance_position) = match user_position {
            Some(pos) => {
                let path = state
                    .get_user_auth_path(pos)
                    .await
                    .map_err(|e| Status::internal(format!("failed to get user auth path: {e}")))?;

                let proto_path = MerklePath {
                    layers: path
                        .into_iter()
                        .map(|siblings| MerklePathLayer {
                            siblings: siblings.iter().map(|c| c.0.to_bytes().to_vec()).collect(),
                        })
                        .collect(),
                };

                (true, Some(proto_path), pos)
            }
            None => (false, None, 0),
        };

        let (asset_registered, is_regulated, asset_path, asset_pos) =
            match (asset_position_opt, asset_status) {
                (Some(pos), Some(regulated)) => {
                    let path = state.get_asset_auth_path(pos).await.map_err(|e| {
                        Status::internal(format!("failed to get asset auth path: {e}"))
                    })?;

                    let proto_path = MerklePath {
                        layers: path
                            .into_iter()
                            .map(|siblings| MerklePathLayer {
                                siblings: siblings
                                    .iter()
                                    .map(|c| c.0.to_bytes().to_vec())
                                    .collect(),
                            })
                            .collect(),
                    };

                    (true, regulated, Some(proto_path), pos)
                }
                _ => (false, false, None, 0),
            };

        tracing::debug!(
            ?address,
            ?asset_id,
            user_registered,
            asset_registered,
            is_regulated,
            compliance_position,
            asset_position = asset_pos,
            "returning compliance merkle proofs"
        );

        let response = ComplianceMerkleProofsResponse {
            user_registered,
            asset_registered,
            is_regulated,
            compliance_path,
            compliance_position,
            asset_path,
            asset_position: asset_pos,
            compliance_anchor: compliance_anchor.0.to_bytes().to_vec(),
            asset_anchor: asset_anchor.0.to_bytes().to_vec(),
        };

        Ok(tonic::Response::new(response))
    }

    #[instrument(skip(self, request))]
    async fn compliance_user_leaf(
        &self,
        request: tonic::Request<ComplianceUserLeafRequest>,
    ) -> Result<tonic::Response<ComplianceUserLeafResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let request = request.into_inner();

        // Parse address (Address)
        let address: Address = request
            .address
            .ok_or_else(|| Status::invalid_argument("missing address"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("could not parse address: {e}")))?;

        // Parse asset_id
        let asset_id: asset::Id = request
            .asset_id
            .ok_or_else(|| Status::invalid_argument("missing asset_id"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("could not parse asset_id: {e}")))?;

        // Look up the user's full leaf from state
        let leaf_opt = state
            .get_user_leaf(&address, asset_id)
            .await
            .map_err(|e| Status::internal(format!("failed to get user leaf: {e}")))?;

        let response = match leaf_opt {
            Some(leaf) => {
                tracing::debug!(?address, ?asset_id, "found user leaf");
                use penumbra_sdk_proto::core::component::compliance::v1 as pb;
                ComplianceUserLeafResponse {
                    is_registered: true,
                    leaf: Some(pb::ComplianceLeaf {
                        address: Some(leaf.address.into()),
                        key: Some(pb::ComplianceViewingKey {
                            inner: leaf.key.0.vartime_compress().0.to_vec(),
                        }),
                        asset_id: Some(leaf.asset_id.into()),
                    }),
                }
            }
            None => {
                tracing::debug!(?address, ?asset_id, "user not registered");
                ComplianceUserLeafResponse {
                    is_registered: false,
                    leaf: None,
                }
            }
        };

        Ok(tonic::Response::new(response))
    }
}
