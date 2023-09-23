//! GRPC service definitions for querying the storage backend directly.

/// Generated proto definitions.
pub mod proto {
    pub mod penumbra {
        pub mod storage {
            pub mod v1alpha1 {
                include!("gen/penumbra.storage.v1alpha1.rs");
                include!("gen/penumbra.storage.v1alpha1.serde.rs");
            }
        }
    }

    // https://github.com/penumbra-zone/penumbra/issues/3038#issuecomment-1722534133
    pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("gen/proto_descriptor.bin.no_lfs");
}
