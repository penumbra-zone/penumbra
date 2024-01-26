pub mod state_key;

#[derive(Clone, Debug, Serialize, Deserialize)]
// #[serde(try_from = "pb::FmdParameters", into = "pb::FmdParameters")]
pub struct Parameters {
    /// Bits of precision.
    pub precision_bits: u8,
    /// The block height at which these parameters became effective.
    pub as_of_block_height: u64,
}

// TODO(erwan): re-enable on second pass
// impl DomainType for FmdParameters {
//     type Proto = pb::FmdParameters;
// }
//
// impl TryFrom<pb::FmdParameters> for FmdParameters {
//     type Error = anyhow::Error;
//
//     fn try_from(msg: pb::FmdParameters) -> Result<Self, Self::Error> {
//         Ok(FmdParameters {
//             precision_bits: msg.precision_bits.try_into()?,
//             as_of_block_height: msg.as_of_block_height,
//         })
//     }
// }
//
// impl From<FmdParameters> for pb::FmdParameters {
//     fn from(params: FmdParameters) -> Self {
//         pb::FmdParameters {
//             precision_bits: u32::from(params.precision_bits),
//             as_of_block_height: params.as_of_block_height,
//         }
//     }
// }

impl Default for Parameters {
    fn default() -> Self {
        Self {
            precision_bits: 0,
            as_of_block_height: 1,
        }
    }
}
