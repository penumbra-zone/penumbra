//! Lookup tables for decryption.
//!
//! Currently, this module just has a naive, in-memory implementation.  In the
//! future, it should have a trait that allows users to plug their own storage
//! system to back the lookup table.

use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc};

use futures::FutureExt;
use parking_lot::Mutex;

/// An error indicating that the [`DecryptionTable`] did not contain a requested
/// discrete logarithm.
///
/// Users of the library should go to great effort to guarantee that this never
/// happens, by bounding the maximum number of ciphertexts aggregated at once
/// relative to the size of the decryption table.
#[derive(thiserror::Error, Debug)]
#[error("requested value not contained in lookup table")]
pub struct TableLookupError {}

/// A (possibly asynchronous) access to a discrete-log lookup table.
///
/// The keys of the table are 32-byte encodings of group elements, and the
/// values are 32-bit integer discrete logarithms.
///
/// Before use, the decryption table should have been initialized with all
/// discrete logarithms up to the maximum possible bitsize that can occur in
/// decryption: `2^{16 + lg(N)}`, where `N` is the number of ciphertexts to
/// aggregate (e.g., aggregating up to 64 ciphertexts requires a table of size
/// `2^{16 + 6} = 2^22`).  The provided `initialize` method will generate the
/// necessary calls to `store` to initialize the table.
pub trait DecryptionTable: Send + Sync {
    /// Look up a 32-bit discrete logarithm by the byte-encoded group element.
    ///
    /// Implementors should return `Ok(None)` on missing keys, and reserve
    /// `Err(e)` for underlying I/O errors.
    fn lookup(
        &self,
        key: [u8; 32],
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<u32>>> + Send + 'static>>;
    /// Store a 32-bit discrete logarithm, indexed by the byte-encoded group element.
    // idea: a rocksdb backing store could store the data in the form
    // (nbits || key, value)
    // where nbits is the bitsize of value
    // then do a linear scan through bitsizes to cut the working set size
    // might not be better than naive caching?
    fn store(
        &self,
        key: [u8; 32],
        value: u32,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>>;

    /// Initialize an empty table.
    ///
    /// This will generate calls to `store` that record all discrete logarithms
    /// up to `2^k`.
    fn initialize(
        &self,
        k: usize,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + '_>> {
        #[allow(non_snake_case)]
        async move {
            let (mut x, mut xB) = (0u32, decaf377::Element::default());
            let B = decaf377::Element::GENERATOR;

            let bound = 1 << k;
            while x < bound {
                self.store(xB.vartime_compress().0, x).await?;
                x += 1;
                xB += B;
            }

            Ok(())
        }
        .boxed()
    }
}

/// A naive, in-memory decryption table for testing.
///
/// Backed by a [`BTreeMap`].
pub struct MockDecryptionTable {
    inner: Arc<Mutex<BTreeMap<[u8; 32], u32>>>,
}

impl Default for MockDecryptionTable {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BTreeMap::default())),
        }
    }
}

impl DecryptionTable for MockDecryptionTable {
    fn lookup(
        &self,
        key: [u8; 32],
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<u32>>> + Send + 'static>> {
        futures::future::ready(Ok(self.inner.lock().get(&key).cloned())).boxed()
    }

    fn store(
        &self,
        key: [u8; 32],
        value: u32,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>> {
        self.inner.lock().insert(key, value);
        futures::future::ready(Ok(())).boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn build_bitsize(bitsize: usize) {
        use std::time::Instant;
        let now = Instant::now();

        let table = MockDecryptionTable::default();
        table
            .initialize(bitsize)
            .await
            .expect("unable to initialize test table");

        let elapsed = now.elapsed();

        println!(
            "Table of bitsize {} took {}s",
            bitsize,
            elapsed.as_secs_f64()
        );
    }

    #[tokio::test]
    #[ignore]
    async fn build_16() {
        build_bitsize(16).await;
    }

    #[tokio::test]
    #[ignore]
    async fn build_21() {
        build_bitsize(21).await;
    }
}
