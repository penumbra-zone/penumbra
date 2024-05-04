use penumbra_proto::custody::v1::{self as pb, AuthorizeResponse};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_with::{formats::Uppercase, hex::Hex};
use tonic::{async_trait, Request, Response, Status};

use crate::{soft_kms, terminal::Terminal, threshold};

mod encryption {
    use anyhow::anyhow;
    use chacha20poly1305::{
        aead::{AeadInPlace, NewAead},
        ChaCha20Poly1305,
    };
    use rand_core::CryptoRngCore;

    /// Represents a password that has been validated for length, and won't cause argon2 errors
    #[derive(Clone, Copy)]
    pub struct Password<'a>(&'a str);

    impl<'a> Password<'a> {
        /// Create a new password, validating its length
        pub fn new(password: &'a str) -> anyhow::Result<Self> {
            anyhow::ensure!(password.len() < argon2::MAX_PWD_LEN, "password too long");
            Ok(Self(password))
        }
    }

    impl<'a> TryFrom<&'a str> for Password<'a> {
        type Error = anyhow::Error;

        fn try_from(value: &'a str) -> Result<Self, Self::Error> {
            Self::new(value)
        }
    }

    // These can be recomputed from the library, at the cost of importing 25 billion traits.
    const SALT_SIZE: usize = 32;
    const TAG_SIZE: usize = 16;
    const KEY_SIZE: usize = 32;

    fn derive_key(salt: &[u8; SALT_SIZE], password: Password<'_>) -> [u8; KEY_SIZE] {
        let mut key = [0u8; KEY_SIZE];
        // The only reason this function should fail is because of incorrect static parameters
        // we've chosen, since we've validated the length of the password.
        argon2::Argon2::hash_password_into(
            &Default::default(),
            password.0.as_bytes(),
            salt,
            &mut key,
        )
        .expect("password hashing should not fail with a small enough password");
        key
    }

    pub fn encrypt(rng: &mut impl CryptoRngCore, password: Password<'_>, data: &[u8]) -> Vec<u8> {
        // The scheme here is that we derive a new salt, used that to derive a new unique key
        // from the password, then store the salt alongside the ciphertext, and its tag.
        // The salt needs to go into the AD section, because we don't want it to be modified,
        // since we're not using a key-committing encryption scheme, and a different key may
        // successfully decrypt the ciphertext.
        let salt = {
            let mut out = [0u8; SALT_SIZE];
            rng.fill_bytes(&mut out);
            out
        };
        let key = derive_key(&salt, password);

        let mut ciphertext = Vec::new();
        ciphertext.extend_from_slice(&[0u8; TAG_SIZE]);
        ciphertext.extend_from_slice(&salt);
        ciphertext.extend_from_slice(&data);
        let tag = ChaCha20Poly1305::new(&key.into())
            .encrypt_in_place_detached(
                &Default::default(),
                &salt,
                &mut ciphertext[TAG_SIZE + SALT_SIZE..],
            )
            .expect("XChaCha20Poly1305 encryption should not fail");
        ciphertext[0..TAG_SIZE].copy_from_slice(&tag);
        ciphertext
    }

    pub fn decrypt(password: Password<'_>, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        anyhow::ensure!(
            data.len() >= TAG_SIZE + SALT_SIZE,
            "failed to decrypt ciphertext"
        );
        let (header, message) = data.split_at(TAG_SIZE + SALT_SIZE);
        let mut message = message.to_owned();
        let tag = &header[..TAG_SIZE];
        let salt = &header[TAG_SIZE..TAG_SIZE + SALT_SIZE];
        let key = derive_key(
            &salt.try_into().expect("salt is the right length"),
            password,
        );
        ChaCha20Poly1305::new(&key.into())
            .decrypt_in_place_detached(&Default::default(), &salt, &mut message, tag.into())
            .map_err(|_| anyhow!("failed to decrypt ciphertext"))?;
        Ok(message)
    }

    #[cfg(test)]
    mod test {
        use rand_core::OsRng;

        use super::*;

        #[test]
        fn test_encryption_decryption_roundtrip() -> anyhow::Result<()> {
            let password = "password".try_into()?;
            let message = b"hello world";
            let encrypted = encrypt(&mut OsRng, password, message);
            let decrypted = decrypt(password, &encrypted)?;
            assert_eq!(decrypted.as_slice(), message);
            Ok(())
        }

        #[test]
        fn test_encryption_fails_with_different_password() -> anyhow::Result<()> {
            let password = "password".try_into()?;
            let message = b"hello world";
            let encrypted = encrypt(&mut OsRng, password, message);
            let decrypted = decrypt("not password".try_into()?, &encrypted);
            assert!(decrypted.is_err());
            Ok(())
        }
    }
}

use encryption::{decrypt, encrypt};

/// The actual inner configuration used for an encrypted configuration.
#[derive(Serialize, Deserialize)]
pub enum InnerConfig {
    SoftKms(soft_kms::Config),
    Threshold(threshold::Config),
}

impl InnerConfig {
    pub fn from_bytes(data: &[u8]) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(data)?)
    }

    pub fn to_bytes(self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(&self)?)
    }
}

/// The configuration for the encrypted custody backend.
///
/// This holds a blob of encrypted data that needs to be further deserialized into another config.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde_as(as = "Hex<Uppercase>")]
    data: Vec<u8>,
}

impl Config {
    /// Create a config from an inner config, with the actual params, and an encryption password.
    pub fn create(password: &str, inner: InnerConfig) -> anyhow::Result<Self> {
        let password = password.try_into()?;
        Ok(Self {
            data: encrypt(&mut OsRng, password, &inner.to_bytes()?),
        })
    }

    fn decrypt(self, password: &str) -> anyhow::Result<InnerConfig> {
        let decrypted_data = decrypt(password.try_into()?, &self.data)?;
        Ok(InnerConfig::from_bytes(&decrypted_data)?)
    }
}

/// Represents a custody service that uses an encrypted configuration.
///
/// This service wraps either the threshold or solo custody service.
pub struct Encrypted {
    inner: Box<dyn pb::custody_service_server::CustodyService>,
}

impl Encrypted {
    /// Create a new encrypted config, using the terminal to ask for a password
    pub async fn new<T: Terminal + Send + Sync + 'static>(
        config: Config,
        terminal: T,
    ) -> anyhow::Result<Self> {
        let password = terminal.get_password().await?;

        let inner = config.decrypt(&password)?;
        let inner: Box<dyn pb::custody_service_server::CustodyService> = match inner {
            InnerConfig::SoftKms(c) => Box::new(soft_kms::SoftKms::new(c)),
            InnerConfig::Threshold(c) => Box::new(threshold::Threshold::new(c, terminal)),
        };
        Ok(Self { inner })
    }
}

#[async_trait]
impl pb::custody_service_server::CustodyService for Encrypted {
    async fn authorize(
        &self,
        request: Request<pb::AuthorizeRequest>,
    ) -> Result<Response<AuthorizeResponse>, Status> {
        self.inner.authorize(request).await
    }

    async fn authorize_validator_definition(
        &self,
        request: Request<pb::AuthorizeValidatorDefinitionRequest>,
    ) -> Result<Response<pb::AuthorizeValidatorDefinitionResponse>, Status> {
        self.inner.authorize_validator_definition(request).await
    }

    async fn authorize_validator_vote(
        &self,
        request: Request<pb::AuthorizeValidatorVoteRequest>,
    ) -> Result<Response<pb::AuthorizeValidatorVoteResponse>, Status> {
        self.inner.authorize_validator_vote(request).await
    }

    async fn export_full_viewing_key(
        &self,
        request: Request<pb::ExportFullViewingKeyRequest>,
    ) -> Result<Response<pb::ExportFullViewingKeyResponse>, Status> {
        self.inner.export_full_viewing_key(request).await
    }

    async fn confirm_address(
        &self,
        request: Request<pb::ConfirmAddressRequest>,
    ) -> Result<Response<pb::ConfirmAddressResponse>, Status> {
        self.inner.confirm_address(request).await
    }
}
