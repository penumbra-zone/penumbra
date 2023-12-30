use anyhow::Context;
use penumbra_proto::{
    util::tendermint_proxy::v1alpha1::tendermint_proxy_service_client::TendermintProxyServiceClient,
    DomainType,
};
use penumbra_transaction::{plan::TransactionPlan, txhash::TransactionId, Transaction};
use penumbra_view::ViewClient;
use std::future::Future;
use tonic::transport::{Channel, ClientTlsConfig};
use tracing::instrument;

use crate::App;

impl App {
    pub async fn build_and_submit_transaction(
        &mut self,
        plan: TransactionPlan,
    ) -> anyhow::Result<TransactionId> {
        let transaction = self.build_transaction(plan).await?;
        self.submit_transaction(transaction).await
    }

    pub fn build_transaction(
        &mut self,
        plan: TransactionPlan,
    ) -> impl Future<Output = anyhow::Result<Transaction>> + '_ {
        println!("building transaction...");
        let start = std::time::Instant::now();
        let tx = penumbra_wallet::build_transaction(
            &self.config.full_viewing_key,
            self.view.as_mut().expect("view service initialized"),
            &mut self.custody,
            plan,
        );
        async move {
            let tx = tx.await?;
            let elapsed = start.elapsed();
            println!(
                "finished proving in {}.{:03} seconds [{} actions, {} proofs, {} bytes]",
                elapsed.as_secs(),
                elapsed.subsec_millis(),
                tx.actions().count(),
                tx.num_proofs(),
                tx.encode_to_vec().len()
            );
            Ok(tx)
        }
    }

    /// Submits a transaction to the network.
    pub async fn submit_transaction(
        &mut self,
        transaction: Transaction,
    ) -> anyhow::Result<TransactionId> {
        println!("broadcasting transaction and awaiting confirmation...");
        let (id, detection_height) = self.view().broadcast_transaction(transaction, true).await?;
        if detection_height != 0 {
            println!(
                "transaction confirmed and detected: {} @ height {}",
                id, detection_height
            );
        } else {
            println!("transaction confirmed and detected: {}", id);
        }
        Ok(id)
    }

    /// Submits a transaction to the network, returning `Ok` as soon as the
    /// transaction has been submitted, rather than waiting for confirmation.
    #[instrument(skip(self, transaction))]
    pub async fn submit_transaction_unconfirmed(
        &mut self,
        transaction: Transaction,
    ) -> anyhow::Result<()> {
        println!("broadcasting transaction without confirmation...");
        self.view()
            .broadcast_transaction(transaction, false)
            .await?;

        Ok(())
    }

    // TODO: why do we need this here but not in the view crate?
    pub async fn pd_channel(&self) -> anyhow::Result<Channel> {
        match self.config.grpc_url.scheme() {
            "http" => Ok(Channel::from_shared(self.config.grpc_url.to_string())?
                .connect()
                .await?),
            "https" => Ok(Channel::from_shared(self.config.grpc_url.to_string())?
                .tls_config(ClientTlsConfig::new())?
                .connect()
                .await?),
            other => Err(anyhow::anyhow!("unknown url scheme {other}"))
                .with_context(|| format!("could not connect to {}", self.config.grpc_url)),
        }
    }

    pub async fn tendermint_proxy_client(
        &self,
    ) -> anyhow::Result<TendermintProxyServiceClient<Channel>> {
        let channel = self.pd_channel().await?;
        Ok(TendermintProxyServiceClient::new(channel))
    }
}
