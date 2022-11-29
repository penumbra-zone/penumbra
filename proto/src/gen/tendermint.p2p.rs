#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetAddress {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub ip: ::prost::alloc::string::String,
    #[prost(uint32, tag="3")]
    pub port: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProtocolVersion {
    #[prost(uint64, tag="1")]
    pub p2p: u64,
    #[prost(uint64, tag="2")]
    pub block: u64,
    #[prost(uint64, tag="3")]
    pub app: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DefaultNodeInfo {
    #[prost(message, optional, tag="1")]
    pub protocol_version: ::core::option::Option<ProtocolVersion>,
    #[prost(string, tag="2")]
    pub default_node_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub listen_addr: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub network: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub version: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="6")]
    pub channels: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub moniker: ::prost::alloc::string::String,
    #[prost(message, optional, tag="8")]
    pub other: ::core::option::Option<DefaultNodeInfoOther>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DefaultNodeInfoOther {
    #[prost(string, tag="1")]
    pub tx_index: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub rpc_address: ::prost::alloc::string::String,
}
