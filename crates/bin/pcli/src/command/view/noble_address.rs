use anyhow::Result;
use futures::future::{BoxFuture, FutureExt};
use penumbra_proto::{
    cosmos::tx::v1beta1::{
        mode_info::{Single, Sum},
        service_client::ServiceClient as CosmosServiceClient,
        AuthInfo as CosmosAuthInfo, BroadcastTxRequest as CosmosBroadcastTxRequest,
        Fee as CosmosFee, ModeInfo, SignerInfo as CosmosSignerInfo, Tx as CosmosTx,
        TxBody as CosmosTxBody,
    },
    noble::forwarding::v1::{ForwardingPubKey, MsgRegisterAccount},
    Message, Name as _,
};
use std::time::Duration;

use penumbra_keys::{address::NobleForwardingAddress, keys::AddressIndex, Address, FullViewingKey};
use tonic::transport::{Channel, ClientTlsConfig};
use url::Url;

#[derive(Debug, clap::Parser)]
pub struct NobleAddressCmd {
    /// The account that should receive forwarded funds.
    #[clap(default_value = "0")]
    account: u32,
    /// The Noble IBC channel to use for forwarding.
    #[clap(long)]
    channel: String,
    /// The Noble node to submit the forwarding registration transaction to.
    #[clap(long)]
    noble_node: Url,
}

impl NobleAddressCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn offline(&self) -> bool {
        true
    }

    pub async fn exec(&self, fvk: &FullViewingKey) -> Result<()> {
        let account = self.account;
        let channel = &self.channel;
        let noble_node = &self.noble_node;

        let next_sequence: u16 =
            get_next_noble_sequence(Some(account), &fvk, &channel, &noble_node).await?;

        let address = get_forwarding_address_for_sequence(next_sequence, Some(account), &fvk);
        let noble_address = address.noble_forwarding_address(&channel);

        println!("next one-time use Noble forwarding address for account {} is: {}\n\nplease deposit funds to this address...\n\nawaiting deposit...\n\n", account, noble_address);

        wait_for_noble_deposit(&noble_node, &noble_address, &address, &channel).await?;
        println!(
                    "💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫💫\n\nregistered Noble forwarding account with address {} to forward to Penumbra address {}...\n\nyour funds should show up in your Penumbra account shortly",
                    noble_address, address
                );

        Ok(())
    }
}

async fn get_next_noble_sequence(
    account: Option<u32>,
    fvk: &FullViewingKey,
    channel: &str,
    noble_node: &Url,
) -> Result<u16> {
    // perform binary search to find the first unused noble sequence number
    // search space (sequence number) is 2 bytes wide
    let left = 0u16;
    let right = 0xffffu16;
    let mid = (left + right) / 2u16;

    // attempt to register midpoint
    _get_next_noble_sequence(left, right, mid, noble_node, channel, fvk, account).await
}

// Helper function to perform recursive binary search
fn _get_next_noble_sequence<'a>(
    left: u16,
    right: u16,
    mid: u16,
    noble_node: &'a Url,
    channel: &'a str,
    fvk: &'a FullViewingKey,
    account: Option<u32>,
) -> BoxFuture<'a, Result<u16>> {
    async move {
        let address = get_forwarding_address_for_sequence(mid, account, fvk);
        let noble_address = address.noble_forwarding_address(channel);
        let noble_res =
            register_noble_forwarding_account(noble_node, &noble_address, &address, channel)
                .await?;
        match noble_res {
            NobleRegistrationResponse::NeedsDeposit => {
                if left == mid || right == mid {
                    // We've iterated as far as we can, the next sequence number
                    // should be the midpoint.
                    return Ok(mid);
                }

                // This means the midpoint has not been registered yet. Search the left-hand
                // side.
                _get_next_noble_sequence(
                    left,
                    mid,
                    (left + mid) / 2,
                    noble_node,
                    channel,
                    fvk,
                    account,
                )
                .await
            }
            NobleRegistrationResponse::Success => {
                // This means the midpoint had a deposit in it waiting for registration.
                // This will "flush" this unregistered address, however the user still wants a new one, so return the midpoint + 1.
                Ok(mid + 1)
            }
            NobleRegistrationResponse::AlreadyRegistered => {
                if left == mid || right == mid {
                    // We've iterated as far as we can, the next sequence number
                    // after the midpoint should be the next available sequence number.
                    return Ok(mid + 1);
                }

                // This means the midpoint has been registered already. Search the right-hand side.
                _get_next_noble_sequence(
                    mid,
                    right,
                    (right + mid) / 2,
                    noble_node,
                    channel,
                    fvk,
                    account,
                )
                .await
            }
        }
    }
    .boxed()
}

fn get_forwarding_address_for_sequence(
    sequence: u16,
    account: Option<u32>,
    fvk: &FullViewingKey,
) -> Address {
    // Noble Randomizer: [0xff; 10] followed by LE16(sequence)
    let mut randomizer: [u8; 12] = [0xff; 12];
    let seq_bytes = sequence.to_le_bytes();
    randomizer[10..].copy_from_slice(&seq_bytes);

    let index = AddressIndex {
        account: account.unwrap_or_default(),
        randomizer,
    };

    let (address, _dtk) = fvk.incoming().payment_address(index.into());

    address
}

async fn register_noble_forwarding_account(
    noble_node: &Url,
    noble_address: &NobleForwardingAddress,
    address: &Address,
    channel: &str,
) -> Result<NobleRegistrationResponse> {
    let mut noble_client = CosmosServiceClient::new(
        Channel::from_shared(noble_node.to_string())?
            .tls_config(ClientTlsConfig::new())?
            .connect()
            .await?,
    );

    let tx = CosmosTx {
        body: Some(CosmosTxBody {
            messages: vec![pbjson_types::Any {
                type_url: MsgRegisterAccount::type_url(),
                value: MsgRegisterAccount {
                    signer: noble_address.to_string(),
                    recipient: address.to_string(),
                    channel: channel.to_string(),
                }
                .encode_to_vec()
                .into(),
            }],
            memo: "".to_string(),
            timeout_height: 0,
            extension_options: vec![],
            non_critical_extension_options: vec![],
        }),
        auth_info: Some(CosmosAuthInfo {
            signer_infos: vec![CosmosSignerInfo {
                public_key: Some(pbjson_types::Any {
                    type_url: ForwardingPubKey::type_url(),
                    value: ForwardingPubKey {
                        key: noble_address.bytes(),
                    }
                    .encode_to_vec()
                    .into(),
                }),
                mode_info: Some(ModeInfo {
                    // SIGN_MODE_DIRECT
                    sum: Some(Sum::Single(Single { mode: 1 })),
                }),
                sequence: 0,
            }],
            fee: Some(CosmosFee {
                amount: vec![],
                gas_limit: 200000u64,
                payer: "".to_string(),
                granter: "".to_string(),
            }),
            tip: None,
        }),
        signatures: vec![vec![]],
    };
    let r = noble_client
        .broadcast_tx(CosmosBroadcastTxRequest {
            tx_bytes: tx.encode_to_vec().into(),
            // sync
            mode: 2,
        })
        .await?
        .into_inner();

    let code = r
        .tx_response
        .ok_or_else(|| anyhow::anyhow!("no tx response"))?
        .code;

    match code {
        9 => Ok(NobleRegistrationResponse::NeedsDeposit),
        0 => Ok(NobleRegistrationResponse::Success),
        19 => Ok(NobleRegistrationResponse::AlreadyRegistered),
        _ => Err(anyhow::anyhow!("unknown response from Noble")),
    }
}

#[derive(Debug, Clone, Copy)]
enum NobleRegistrationResponse {
    NeedsDeposit,
    Success,
    AlreadyRegistered,
}

async fn wait_for_noble_deposit(
    noble_node: &Url,
    noble_address: &NobleForwardingAddress,
    address: &Address,
    channel: &str,
) -> Result<()> {
    // Use exponential backoff to attempt to register the noble address
    // until it's successful.
    let max_interval = Duration::from_secs(8);
    let mut current_interval = Duration::from_secs(1);

    loop {
        let noble_res =
            register_noble_forwarding_account(noble_node, &noble_address, &address, channel)
                .await?;
        match noble_res {
            NobleRegistrationResponse::Success => {
                return Ok(());
            }
            NobleRegistrationResponse::AlreadyRegistered => {
                return Ok(());
            }
            NobleRegistrationResponse::NeedsDeposit => {
                // Wait for a bit and try again.
                tokio::time::sleep(current_interval).await;
                current_interval = std::cmp::min(max_interval, current_interval * 2);
            }
        }
    }
}
