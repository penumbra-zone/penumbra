use std::collections::BTreeMap;

use anyhow::Context;
use async_trait::async_trait;
use futures::StreamExt;
use ibc_proto::cosmos::bank::v1beta1::{
    query_server::Query as BankQuery, QueryAllBalancesRequest, QueryAllBalancesResponse,
    QueryBalanceRequest, QueryBalanceResponse, QueryParamsRequest, QueryParamsResponse,
    QueryTotalSupplyRequest, QueryTotalSupplyResponse,
};
use ibc_proto::cosmos::bank::v1beta1::{
    QueryDenomMetadataByQueryStringRequest, QueryDenomMetadataByQueryStringResponse,
    QueryDenomMetadataRequest, QueryDenomMetadataResponse, QueryDenomOwnersByQueryRequest,
    QueryDenomOwnersByQueryResponse, QueryDenomOwnersRequest, QueryDenomOwnersResponse,
    QueryDenomsMetadataRequest, QueryDenomsMetadataResponse, QuerySendEnabledRequest,
    QuerySendEnabledResponse, QuerySpendableBalanceByDenomRequest,
    QuerySpendableBalanceByDenomResponse, QuerySpendableBalancesRequest,
    QuerySpendableBalancesResponse, QuerySupplyOfRequest, QuerySupplyOfResponse,
};
use penumbra_sdk_asset::asset::{self, Metadata};
use penumbra_sdk_ibc::component::state_key as ibc_state_key;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::StateReadProto as _;
use tracing::instrument;

use crate::component::AssetRegistryRead as _;
use crate::state_key;

use super::Server;

#[async_trait]
impl BankQuery for Server {
    /// Returns the total supply for all IBC assets.
    /// Internally-minted assets (Penumbra tokens, LP tokens, delegation tokens, etc.)
    /// are also included but the supplies are will only reflect what has been transferred out.
    ///
    /// TODO: Implement a way to fetch the total supply for these assets.
    /// TODO: implement pagination
    #[instrument(skip(self, _request))]
    async fn total_supply(
        &self,
        _request: tonic::Request<QueryTotalSupplyRequest>,
    ) -> Result<tonic::Response<QueryTotalSupplyResponse>, tonic::Status> {
        let snapshot = self.storage.latest_snapshot();

        // Find every non-IBC known asset
        let s = snapshot.prefix(state_key::denom_metadata_by_asset::prefix());
        let mut total_supply = s
            .filter_map(move |i: anyhow::Result<(String, Metadata)>| async move {
                if i.is_err() {
                    return Some(Err(i.context("bad denom in state").err().unwrap()));
                }
                let (_key, denom_metadata) = i.expect("should not be an error");

                // Return a hardcoded 0 supply for now
                Some(Ok((denom_metadata, Amount::from(0u32))))
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .into_iter()
            .collect::<BTreeMap<_, _>>();

        let s = snapshot.prefix(ibc_state_key::ics20_value_balance::prefix());
        let ibc_amounts = s
            .filter_map(move |i: anyhow::Result<(String, Amount)>| async move {
                if i.is_err() {
                    return Some(Err(i.context("bad amount in state").err().unwrap()));
                }
                let (key, amount) = i.expect("should not be an error");

                // Extract the asset ID from the key
                let asset_id = key.split('/').last();
                if asset_id.is_none() {
                    return Some(Err(asset_id
                        .context("bad IBC ics20 value balance key in state")
                        .err()
                        .unwrap()));
                }
                let asset_id = asset_id.expect("should not be an error");

                // Parse the asset ID
                let asset_id = asset_id.parse::<asset::Id>();
                if asset_id.is_err() {
                    return Some(Err(asset_id
                        .context("invalid IBC ics20 value balance asset ID in state")
                        .err()
                        .unwrap()));
                }
                let asset_id = asset_id.expect("should not be an error");

                Some(Ok((asset_id, amount)))
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        // Fetch the denoms associated with the IBC asset IDs
        for (asset_id, amount) in ibc_amounts {
            let denom_metadata = snapshot.denom_metadata_by_asset(&asset_id).await;
            if denom_metadata.is_none() {
                // This is likely an NFT asset that is intentionally
                // not registered, so it is fine to exclude from the output.
                continue;
            }
            let denom_metadata = denom_metadata.expect("should not be an error");

            // Add to the total supply seen for this denom.
            total_supply
                .entry(denom_metadata)
                .and_modify(|a| *a += amount)
                .or_insert(amount);
        }

        Ok(tonic::Response::new(QueryTotalSupplyResponse {
            // Pagination disabled for now
            pagination: None,
            supply: total_supply
                .into_iter()
                .map(
                    |(denom_metadata, amount)| ibc_proto::cosmos::base::v1beta1::Coin {
                        denom: denom_metadata.to_string(),
                        amount: amount.to_string(),
                    },
                )
                .collect::<Vec<ibc_proto::cosmos::base::v1beta1::Coin>>(),
        }))
    }

    async fn params(
        &self,
        _: tonic::Request<QueryParamsRequest>,
    ) -> std::result::Result<tonic::Response<QueryParamsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn balance(
        &self,
        _: tonic::Request<QueryBalanceRequest>,
    ) -> std::result::Result<tonic::Response<QueryBalanceResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented(
            "not implemented, penumbra is a shielded chain",
        ))
    }

    async fn all_balances(
        &self,
        _: tonic::Request<QueryAllBalancesRequest>,
    ) -> std::result::Result<tonic::Response<QueryAllBalancesResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented(
            "not implemented, penumbra is a shielded chain",
        ))
    }

    async fn spendable_balances(
        &self,
        _: tonic::Request<QuerySpendableBalancesRequest>,
    ) -> std::result::Result<tonic::Response<QuerySpendableBalancesResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented(
            "not implemented, penumbra is a shielded chain",
        ))
    }

    async fn spendable_balance_by_denom(
        &self,
        _: tonic::Request<QuerySpendableBalanceByDenomRequest>,
    ) -> std::result::Result<tonic::Response<QuerySpendableBalanceByDenomResponse>, tonic::Status>
    {
        Err(tonic::Status::unimplemented(
            "not implemented, penumbra is a shielded chain",
        ))
    }

    async fn supply_of(
        &self,
        _: tonic::Request<QuerySupplyOfRequest>,
    ) -> std::result::Result<tonic::Response<QuerySupplyOfResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn denom_metadata(
        &self,
        _: tonic::Request<QueryDenomMetadataRequest>,
    ) -> std::result::Result<tonic::Response<QueryDenomMetadataResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn denoms_metadata(
        &self,
        _: tonic::Request<QueryDenomsMetadataRequest>,
    ) -> std::result::Result<tonic::Response<QueryDenomsMetadataResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn denom_owners(
        &self,
        _: tonic::Request<QueryDenomOwnersRequest>,
    ) -> std::result::Result<tonic::Response<QueryDenomOwnersResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn send_enabled(
        &self,
        _: tonic::Request<QuerySendEnabledRequest>,
    ) -> std::result::Result<tonic::Response<QuerySendEnabledResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn denom_metadata_by_query_string(
        &self,
        _: tonic::Request<QueryDenomMetadataByQueryStringRequest>,
    ) -> Result<tonic::Response<QueryDenomMetadataByQueryStringResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn denom_owners_by_query(
        &self,
        _: tonic::Request<QueryDenomOwnersByQueryRequest>,
    ) -> Result<tonic::Response<QueryDenomOwnersByQueryResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}
