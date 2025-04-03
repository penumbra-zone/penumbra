use std::time::Duration;

use anyhow::anyhow;
use ledger_lib::{
    info::AppInfo, Device as _, Exchange, Filters, LedgerHandle, LedgerProvider, Transport as _,
    DEFAULT_TIMEOUT,
};
use ledger_proto::ApduHeader;
use penumbra_sdk_keys::{keys::AddressIndex, Address, FullViewingKey};
use penumbra_sdk_proto::DomainType as _;
use penumbra_sdk_transaction::{txhash::EffectHash, AuthorizationData, TransactionPlan};

fn is_penumbra_app(info: &AppInfo) -> anyhow::Result<()> {
    if info.name != "Penumbra" {
        anyhow::bail!(
            "unknown app: {}. Make sure to open the Penumbra app on your device.",
            &info.name
        );
    }
    Ok(())
}

/// Necessary because an extra byte is needed to optimize the case where there's no randomizer.
///
/// c.f. https://github.com/Zondax/ledger-penumbra-js/blob/d0af0e447d73de9050a258d80db8082e32734046/src/app.ts#L272
fn address_index_to_weird_bytes(index: AddressIndex) -> [u8; 17] {
    let mut out = [0u8; 17];
    out[..4].copy_from_slice(&index.account.to_le_bytes());
    out[4] = u8::from(index.randomizer != [0u8; 12]);
    out[5..].copy_from_slice(&index.randomizer);
    out
}

fn vec_with_fixed_derivation_path(capacity: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(12 + capacity);
    out.extend_from_slice(&u32::to_le_bytes(0x8000_002C));
    // :)
    out.extend_from_slice(&u32::to_le_bytes(0x8000_1984));
    out.extend_from_slice(&u32::to_le_bytes(0x8000_0000));
    out
}

fn check_error_code(code: u16) -> anyhow::Result<()> {
    // https://github.com/Zondax/ledger-js/blob/58248aa02ebfe65f5e0e853f3dca66f60c95eacf/src/consts.ts.
    match code {
        0x9000 => Ok(()),
        0x0001 => Err(anyhow!("U2F: Unknown")),
        0x0002 => Err(anyhow!("U2F: Bad request")),
        0x0003 => Err(anyhow!("U2F: Configuration unsupported")),
        0x0004 => Err(anyhow!("U2F: Device Ineligible")),
        0x0005 => Err(anyhow!("U2F: Timeout")),
        0x000E => Err(anyhow!("Timeout")),
        0x5102 => Err(anyhow!("Not Enough Space")),
        0x5501 => Err(anyhow!("User Refused on Device")),
        0x5515 => Err(anyhow!("Device Locked")),
        0x6300 => Err(anyhow!("GP Authentication Failed")),
        0x63C0 => Err(anyhow!("PIN Remaining Attempts")),
        0x6400 => Err(anyhow!("Execution Error")),
        0x6611 => Err(anyhow!("Device Not Onboarded (Secondary)")),
        0x662E => Err(anyhow!("Custom Image Empty")),
        0x662F => Err(anyhow!("Custom Image Bootloader Error")),
        0x6700 => Err(anyhow!("Wrong Length")),
        0x6800 => Err(anyhow!("Missing Critical Parameter")),
        0x6802 => Err(anyhow!("Error deriving keys")),
        0x6981 => Err(anyhow!("Command Incompatible with File Structure")),
        0x6982 => Err(anyhow!("Empty Buffer")),
        0x6983 => Err(anyhow!("Output buffer too small")),
        0x6984 => Err(anyhow!("Data is invalid")),
        0x6985 => Err(anyhow!("Conditions of Use Not Satisfied")),
        0x6986 => Err(anyhow!("Transaction rejected")),
        0x6A80 => Err(anyhow!("Bad key handle")),
        0x6A84 => Err(anyhow!("Not Enough Memory Space")),
        0x6A88 => Err(anyhow!("Referenced Data Not Found")),
        0x6A89 => Err(anyhow!("File Already Exists")),
        0x6B00 => Err(anyhow!("Invalid P1/P2")),
        0x6D00 => Err(anyhow!("Instruction not supported")),
        0x6D02 => Err(anyhow!("Unknown APDU")),
        0x6D07 => Err(anyhow!("Device Not Onboarded")),
        0x6E00 => Err(anyhow!("CLA Not Supported")),
        0x6E01 => Err(anyhow!("App does not seem to be open")),
        0x6F00 => Err(anyhow!("Unknown error")),
        0x6F01 => Err(anyhow!("Sign/verify error")),
        0x6F42 => Err(anyhow!("Licensing Error")),
        0x6FAA => Err(anyhow!("Device Halted")),
        0x9001 => Err(anyhow!("Device is busy")),
        0x9240 => Err(anyhow!("Memory Problem")),
        0x9400 => Err(anyhow!("No EF Selected")),
        0x9402 => Err(anyhow!("Invalid Offset")),
        0x9404 => Err(anyhow!("File Not Found")),
        0x9408 => Err(anyhow!("Inconsistent File")),
        0x9484 => Err(anyhow!("Algorithm Not Supported")),
        0x9485 => Err(anyhow!("Invalid KCV")),
        0x9802 => Err(anyhow!("Code Not Initialized")),
        0x9804 => Err(anyhow!("Access Condition Not Fulfilled")),
        0x9808 => Err(anyhow!("Contradiction with Secret Code Status")),
        0x9810 => Err(anyhow!("Contradiction Invalidation")),
        0x9840 => Err(anyhow!("Code Blocked")),
        0x9850 => Err(anyhow!("Maximum Value Reached")),
        _ => Err(anyhow!("Unknown transport error")),
    }
}

/// All responses in this particular app follow a common decoding scheme.
///
/// This wraps this scheme, providing a nicer interface, returning `anyhow::Result`,
/// instead of using the somewhat limited [`ledger_proto::ApduError`] type.
struct GenericResponse {
    data: Vec<u8>,
}

impl GenericResponse {
    fn payload(&self) -> anyhow::Result<&'_ [u8]> {
        // c.f. https://github.com/Zondax/ledger-js/blob/58248aa02ebfe65f5e0e853f3dca66f60c95eacf/src/common.ts#L41.
        if self.data.len() < 2 {
            anyhow::bail!("insufficient payload length");
        }
        let payload_end = self.data.len() - 2;
        let code = u16::from_be_bytes(
            self.data[payload_end..]
                .try_into()
                .expect("slice should have length 2"),
        );
        // #golang
        if let Err(e) = check_error_code(code) {
            // When an error happens, the rest of the payload is an additional message.
            // This should be ASCII (and thus UTF-8), but we can just ignore
            // bad characters, using [`String::from_utf8_lossy`];
            return Err(e.context(String::from_utf8_lossy(&self.data[..payload_end]).to_string()));
        }
        Ok(&self.data[..payload_end])
    }
}

pub struct Device {
    handle: LedgerHandle,
    buf: [u8; 256],
}

impl Device {
    pub async fn connect_to_first() -> anyhow::Result<Self> {
        let mut provider = LedgerProvider::init().await;
        let device_list = provider.list(Filters::Any).await?;

        // NOTE: Should we do more than just pick the first device?
        let Some(device_info) = device_list.into_iter().next() else {
            anyhow::bail!("No ledger devices found.");
        };

        tracing::debug!(?device_info, "found ledger device");

        let mut handle = provider.connect(device_info).await?;

        let info = handle.app_info(DEFAULT_TIMEOUT).await?;
        is_penumbra_app(&info)?;

        tracing::debug!(?info, "connected to ledger device");

        Ok(Self {
            handle,
            buf: [0u8; 256],
        })
    }

    async fn request(
        &mut self,
        header: ApduHeader,
        data: &[u8],
    ) -> anyhow::Result<GenericResponse> {
        let req_len = 5 + data.len();
        // For a better error message.
        assert!(req_len <= self.buf.len(), "request payload too large");
        self.buf[0] = header.cla;
        self.buf[1] = header.ins;
        self.buf[2] = header.p1;
        self.buf[3] = header.p2;
        // For empty data, we don't write the length at all.
        if !data.is_empty() {
            self.buf[4] = data.len().try_into().expect("data length should be < 256");
            self.buf[5..req_len].copy_from_slice(data);
        }

        let out = self
            .handle
            .exchange(&self.buf[..req_len], Duration::MAX)
            .await?;
        Ok(GenericResponse { data: out })
    }

    pub async fn get_fvk(&mut self) -> anyhow::Result<FullViewingKey> {
        // https://github.com/Zondax/ledger-penumbra/blob/9f57b82ad3b843bc18e22ba841f971659bcd0fe8/docs/APDUSPEC.md#ins_get_fvk
        let header = ApduHeader {
            cla: 0x80,
            ins: 0x03,
            p1: 0,
            p2: 0,
        };
        let mut req = vec_with_fixed_derivation_path(17);
        // The request requires an address index which doesn't actually influence the result.
        req.extend_from_slice(&[0u8; 17]);
        tracing::debug!("sending FVK request");
        let rsp = self.request(header, &req).await?;
        let fvk = FullViewingKey::try_from(rsp.payload()?)?;
        Ok(fvk)
    }

    pub async fn confirm_addr(&mut self, index: AddressIndex) -> anyhow::Result<Address> {
        // https://github.com/Zondax/ledger-penumbra/blob/9f57b82ad3b843bc18e22ba841f971659bcd0fe8/docs/APDUSPEC.md#ins_get_addr        todo!()
        let header = ApduHeader {
            cla: 0x80,
            ins: 0x01,
            // We want the user to confirm the address.
            p1: 1,
            p2: 0,
        };
        let mut req = vec_with_fixed_derivation_path(17);
        // The request requires an address index which doesn't actually influence the result.
        req.extend_from_slice(&address_index_to_weird_bytes(index));
        tracing::debug!(?index, "sending confirm address request");
        let rsp = self.request(header, &req).await?;
        let addr = Address::try_from(rsp.payload()?)?;
        Ok(addr)
    }

    pub async fn authorize(&mut self, plan: TransactionPlan) -> anyhow::Result<AuthorizationData> {
        // c.f. https://github.com/Zondax/ledger-penumbra-js/blob/d0af0e447d73de9050a258d80db8082e32734046/src/app.ts#L116
        let plan_bytes = plan.encode_to_vec();

        let start = vec_with_fixed_derivation_path(0);

        let mut response = self
            .request(
                ApduHeader {
                    cla: 0x80,
                    ins: 0x02,
                    p1: 0,
                    p2: 0,
                },
                &start,
            )
            .await?;

        let mut chunks = plan_bytes.chunks(250).peekable();
        while let Some(chunk) = chunks.next() {
            let is_last = chunks.peek().is_none();
            response = self
                .request(
                    ApduHeader {
                        cla: 0x80,
                        ins: 0x02,
                        p1: if is_last { 2 } else { 1 },
                        p2: 0,
                    },
                    chunk,
                )
                .await?;
        }

        let response_data = response.payload()?;
        if response_data.len() != 64 + 2 + 2 {
            anyhow::bail!("unexpected signing response");
        }
        let mut auth_data = AuthorizationData {
            effect_hash: Some(EffectHash(response_data[..64].try_into()?)),
            ..Default::default()
        };
        let spend_auth_count: u8 =
            u16::from_le_bytes(response_data[64..66].try_into()?).try_into()?;
        let delegator_auth_count: u8 =
            u16::from_le_bytes(response_data[66..68].try_into()?).try_into()?;

        for i in 0..spend_auth_count {
            response = self
                .request(
                    ApduHeader {
                        cla: 0x80,
                        ins: 0x05,
                        p1: i,
                        p2: 0,
                    },
                    &[],
                )
                .await?;
            auth_data.spend_auths.push(response.payload()?.try_into()?);
        }
        for i in 0..delegator_auth_count {
            response = self
                .request(
                    ApduHeader {
                        cla: 0x80,
                        ins: 0x06,
                        p1: i,
                        p2: 0,
                    },
                    &[],
                )
                .await?;
            auth_data
                .delegator_vote_auths
                .push(response.payload()?.try_into()?);
        }

        Ok(auth_data)
    }
}
