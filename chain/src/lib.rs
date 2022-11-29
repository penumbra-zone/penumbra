mod epoch;
mod known_assets;
mod note_source;
mod view;

pub mod app_hash;
pub mod genesis;
pub mod params;
pub mod quarantined;
pub(crate) mod state_key;
pub mod sync;

pub use app_hash::{AppHash, AppHashRead, PENUMBRA_COMMITMENT_PREFIX, PENUMBRA_PROOF_SPECS};
pub use epoch::Epoch;
pub use known_assets::KnownAssets;
pub use note_source::NoteSource;
pub use sync::{AnnotatedNotePayload, CompactBlock};
pub use view::{StateReadExt, StateWriteExt};

/// Hardcoded test data.
pub mod test {
    /// This address is for test purposes, allocations were added beginning with
    /// the 016-Pandia testnet.
    pub const TEST_SEED_PHRASE: &str = "benefit cherry cannon tooth exhibit law avocado spare tooth that amount pumpkin scene foil tape mobile shine apology add crouch situate sun business explain";

    /// These addresses both correspond to the test wallet above.
    pub const TEST_ADDRESS_0: &str = "penumbrav2t13vh0fkf3qkqjacpm59g23ufea9n5us45e4p5h6hty8vg73r2t8g5l3kynad87u0n9eragf3hhkgkhqe5vhngq2cw493k48c9qg9ms4epllcmndd6ly4v4dw2jcnxaxzjqnlvnw";
    /// These addresses both correspond to the test wallet above.
    pub const TEST_ADDRESS_1: &str = "penumbrav2t1gl609fq6xzjcqn3hz3crysw2s0nkt330lyhaq403ztmrm3yygsgdklt9uxfs0gedwp6sypp5k5ln9t62lvs9t0a990q832wnxak8r939g5u6uz5aessd8saxvv7ewlz4hhqnws";
}
