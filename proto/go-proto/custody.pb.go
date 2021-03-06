// Code generated by protoc-gen-go. DO NOT EDIT.
// versions:
// 	protoc-gen-go v1.28.0
// 	protoc        v3.19.4
// source: custody.proto

package go_proto

import (
	protoreflect "google.golang.org/protobuf/reflect/protoreflect"
	protoimpl "google.golang.org/protobuf/runtime/protoimpl"
	reflect "reflect"
	sync "sync"
)

const (
	// Verify that this generated code is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(20 - protoimpl.MinVersion)
	// Verify that runtime/protoimpl is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(protoimpl.MaxVersion - 20)
)

type AuthorizeRequest struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	// The transaction plan to authorize.
	Plan *TransactionPlan `protobuf:"bytes,1,opt,name=plan,proto3" json:"plan,omitempty"`
	// Identifies the FVK (and hence the spend authorization key) to use for signing.
	FvkHash *FullViewingKeyHash `protobuf:"bytes,2,opt,name=fvk_hash,json=fvkHash,proto3" json:"fvk_hash,omitempty"`
}

func (x *AuthorizeRequest) Reset() {
	*x = AuthorizeRequest{}
	if protoimpl.UnsafeEnabled {
		mi := &file_custody_proto_msgTypes[0]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *AuthorizeRequest) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*AuthorizeRequest) ProtoMessage() {}

func (x *AuthorizeRequest) ProtoReflect() protoreflect.Message {
	mi := &file_custody_proto_msgTypes[0]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use AuthorizeRequest.ProtoReflect.Descriptor instead.
func (*AuthorizeRequest) Descriptor() ([]byte, []int) {
	return file_custody_proto_rawDescGZIP(), []int{0}
}

func (x *AuthorizeRequest) GetPlan() *TransactionPlan {
	if x != nil {
		return x.Plan
	}
	return nil
}

func (x *AuthorizeRequest) GetFvkHash() *FullViewingKeyHash {
	if x != nil {
		return x.FvkHash
	}
	return nil
}

var File_custody_proto protoreflect.FileDescriptor

var file_custody_proto_rawDesc = []byte{
	0x0a, 0x0d, 0x63, 0x75, 0x73, 0x74, 0x6f, 0x64, 0x79, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12,
	0x10, 0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2e, 0x63, 0x75, 0x73, 0x74, 0x6f, 0x64,
	0x79, 0x1a, 0x11, 0x74, 0x72, 0x61, 0x6e, 0x73, 0x61, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x2e, 0x70,
	0x72, 0x6f, 0x74, 0x6f, 0x1a, 0x0c, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x2e, 0x70, 0x72, 0x6f,
	0x74, 0x6f, 0x1a, 0x0b, 0x73, 0x74, 0x61, 0x6b, 0x65, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x1a,
	0x09, 0x69, 0x62, 0x63, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x22, 0x8d, 0x01, 0x0a, 0x10, 0x41,
	0x75, 0x74, 0x68, 0x6f, 0x72, 0x69, 0x7a, 0x65, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x12,
	0x39, 0x0a, 0x04, 0x70, 0x6c, 0x61, 0x6e, 0x18, 0x01, 0x20, 0x01, 0x28, 0x0b, 0x32, 0x25, 0x2e,
	0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2e, 0x74, 0x72, 0x61, 0x6e, 0x73, 0x61, 0x63,
	0x74, 0x69, 0x6f, 0x6e, 0x2e, 0x54, 0x72, 0x61, 0x6e, 0x73, 0x61, 0x63, 0x74, 0x69, 0x6f, 0x6e,
	0x50, 0x6c, 0x61, 0x6e, 0x52, 0x04, 0x70, 0x6c, 0x61, 0x6e, 0x12, 0x3e, 0x0a, 0x08, 0x66, 0x76,
	0x6b, 0x5f, 0x68, 0x61, 0x73, 0x68, 0x18, 0x02, 0x20, 0x01, 0x28, 0x0b, 0x32, 0x23, 0x2e, 0x70,
	0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2e, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x2e, 0x46,
	0x75, 0x6c, 0x6c, 0x56, 0x69, 0x65, 0x77, 0x69, 0x6e, 0x67, 0x4b, 0x65, 0x79, 0x48, 0x61, 0x73,
	0x68, 0x52, 0x07, 0x66, 0x76, 0x6b, 0x48, 0x61, 0x73, 0x68, 0x32, 0x6b, 0x0a, 0x0f, 0x43, 0x75,
	0x73, 0x74, 0x6f, 0x64, 0x79, 0x50, 0x72, 0x6f, 0x74, 0x6f, 0x63, 0x6f, 0x6c, 0x12, 0x58, 0x0a,
	0x09, 0x41, 0x75, 0x74, 0x68, 0x6f, 0x72, 0x69, 0x7a, 0x65, 0x12, 0x22, 0x2e, 0x70, 0x65, 0x6e,
	0x75, 0x6d, 0x62, 0x72, 0x61, 0x2e, 0x63, 0x75, 0x73, 0x74, 0x6f, 0x64, 0x79, 0x2e, 0x41, 0x75,
	0x74, 0x68, 0x6f, 0x72, 0x69, 0x7a, 0x65, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x1a, 0x27,
	0x2e, 0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2e, 0x74, 0x72, 0x61, 0x6e, 0x73, 0x61,
	0x63, 0x74, 0x69, 0x6f, 0x6e, 0x2e, 0x41, 0x75, 0x74, 0x68, 0x6f, 0x72, 0x69, 0x7a, 0x61, 0x74,
	0x69, 0x6f, 0x6e, 0x44, 0x61, 0x74, 0x61, 0x42, 0x32, 0x5a, 0x30, 0x67, 0x69, 0x74, 0x68, 0x75,
	0x62, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2d, 0x7a,
	0x6f, 0x6e, 0x65, 0x2f, 0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2f, 0x70, 0x72, 0x6f,
	0x74, 0x6f, 0x2f, 0x67, 0x6f, 0x2d, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x62, 0x06, 0x70, 0x72, 0x6f,
	0x74, 0x6f, 0x33,
}

var (
	file_custody_proto_rawDescOnce sync.Once
	file_custody_proto_rawDescData = file_custody_proto_rawDesc
)

func file_custody_proto_rawDescGZIP() []byte {
	file_custody_proto_rawDescOnce.Do(func() {
		file_custody_proto_rawDescData = protoimpl.X.CompressGZIP(file_custody_proto_rawDescData)
	})
	return file_custody_proto_rawDescData
}

var file_custody_proto_msgTypes = make([]protoimpl.MessageInfo, 1)
var file_custody_proto_goTypes = []interface{}{
	(*AuthorizeRequest)(nil),   // 0: penumbra.custody.AuthorizeRequest
	(*TransactionPlan)(nil),    // 1: penumbra.transaction.TransactionPlan
	(*FullViewingKeyHash)(nil), // 2: penumbra.crypto.FullViewingKeyHash
	(*AuthorizationData)(nil),  // 3: penumbra.transaction.AuthorizationData
}
var file_custody_proto_depIdxs = []int32{
	1, // 0: penumbra.custody.AuthorizeRequest.plan:type_name -> penumbra.transaction.TransactionPlan
	2, // 1: penumbra.custody.AuthorizeRequest.fvk_hash:type_name -> penumbra.crypto.FullViewingKeyHash
	0, // 2: penumbra.custody.CustodyProtocol.Authorize:input_type -> penumbra.custody.AuthorizeRequest
	3, // 3: penumbra.custody.CustodyProtocol.Authorize:output_type -> penumbra.transaction.AuthorizationData
	3, // [3:4] is the sub-list for method output_type
	2, // [2:3] is the sub-list for method input_type
	2, // [2:2] is the sub-list for extension type_name
	2, // [2:2] is the sub-list for extension extendee
	0, // [0:2] is the sub-list for field type_name
}

func init() { file_custody_proto_init() }
func file_custody_proto_init() {
	if File_custody_proto != nil {
		return
	}
	file_transaction_proto_init()
	file_crypto_proto_init()
	file_stake_proto_init()
	file_ibc_proto_init()
	if !protoimpl.UnsafeEnabled {
		file_custody_proto_msgTypes[0].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*AuthorizeRequest); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
	}
	type x struct{}
	out := protoimpl.TypeBuilder{
		File: protoimpl.DescBuilder{
			GoPackagePath: reflect.TypeOf(x{}).PkgPath(),
			RawDescriptor: file_custody_proto_rawDesc,
			NumEnums:      0,
			NumMessages:   1,
			NumExtensions: 0,
			NumServices:   1,
		},
		GoTypes:           file_custody_proto_goTypes,
		DependencyIndexes: file_custody_proto_depIdxs,
		MessageInfos:      file_custody_proto_msgTypes,
	}.Build()
	File_custody_proto = out.File
	file_custody_proto_rawDesc = nil
	file_custody_proto_goTypes = nil
	file_custody_proto_depIdxs = nil
}
