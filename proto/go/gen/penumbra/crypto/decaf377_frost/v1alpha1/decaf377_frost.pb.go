// Code generated by protoc-gen-go. DO NOT EDIT.
// versions:
// 	protoc-gen-go v1.31.0
// 	protoc        (unknown)
// source: penumbra/crypto/decaf377_frost/v1alpha1/decaf377_frost.proto

package decaf377_frostv1alpha1

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

// A commitment to a polynomial, as a list of group elements.
type VerifiableSecretSharingCommitment struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	// Each of these bytes should be the serialization of a group element.
	Elements [][]byte `protobuf:"bytes,1,rep,name=elements,proto3" json:"elements,omitempty"`
}

func (x *VerifiableSecretSharingCommitment) Reset() {
	*x = VerifiableSecretSharingCommitment{}
	if protoimpl.UnsafeEnabled {
		mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[0]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *VerifiableSecretSharingCommitment) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*VerifiableSecretSharingCommitment) ProtoMessage() {}

func (x *VerifiableSecretSharingCommitment) ProtoReflect() protoreflect.Message {
	mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[0]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use VerifiableSecretSharingCommitment.ProtoReflect.Descriptor instead.
func (*VerifiableSecretSharingCommitment) Descriptor() ([]byte, []int) {
	return file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescGZIP(), []int{0}
}

func (x *VerifiableSecretSharingCommitment) GetElements() [][]byte {
	if x != nil {
		return x.Elements
	}
	return nil
}

// The public package sent in round 1 of the DKG protocol.
type DKGRound1Package struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	// A commitment to the polynomial for secret sharing.
	Commitment *VerifiableSecretSharingCommitment `protobuf:"bytes,1,opt,name=commitment,proto3" json:"commitment,omitempty"`
	// A proof of knowledge of the underlying secret being shared.
	ProofOfKnowledge []byte `protobuf:"bytes,2,opt,name=proof_of_knowledge,json=proofOfKnowledge,proto3" json:"proof_of_knowledge,omitempty"`
}

func (x *DKGRound1Package) Reset() {
	*x = DKGRound1Package{}
	if protoimpl.UnsafeEnabled {
		mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[1]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *DKGRound1Package) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*DKGRound1Package) ProtoMessage() {}

func (x *DKGRound1Package) ProtoReflect() protoreflect.Message {
	mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[1]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use DKGRound1Package.ProtoReflect.Descriptor instead.
func (*DKGRound1Package) Descriptor() ([]byte, []int) {
	return file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescGZIP(), []int{1}
}

func (x *DKGRound1Package) GetCommitment() *VerifiableSecretSharingCommitment {
	if x != nil {
		return x.Commitment
	}
	return nil
}

func (x *DKGRound1Package) GetProofOfKnowledge() []byte {
	if x != nil {
		return x.ProofOfKnowledge
	}
	return nil
}

// A share of the final signing key.
type SigningShare struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	// These bytes should be a valid scalar.
	Scalar []byte `protobuf:"bytes,1,opt,name=scalar,proto3" json:"scalar,omitempty"`
}

func (x *SigningShare) Reset() {
	*x = SigningShare{}
	if protoimpl.UnsafeEnabled {
		mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[2]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *SigningShare) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*SigningShare) ProtoMessage() {}

func (x *SigningShare) ProtoReflect() protoreflect.Message {
	mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[2]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use SigningShare.ProtoReflect.Descriptor instead.
func (*SigningShare) Descriptor() ([]byte, []int) {
	return file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescGZIP(), []int{2}
}

func (x *SigningShare) GetScalar() []byte {
	if x != nil {
		return x.Scalar
	}
	return nil
}

// The per-participant package sent in round 2 of the DKG protocol.
type DKGRound2Package struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	// This is the share we're sending to that participant.
	SigningShare *SigningShare `protobuf:"bytes,1,opt,name=signing_share,json=signingShare,proto3" json:"signing_share,omitempty"`
}

func (x *DKGRound2Package) Reset() {
	*x = DKGRound2Package{}
	if protoimpl.UnsafeEnabled {
		mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[3]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *DKGRound2Package) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*DKGRound2Package) ProtoMessage() {}

func (x *DKGRound2Package) ProtoReflect() protoreflect.Message {
	mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[3]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use DKGRound2Package.ProtoReflect.Descriptor instead.
func (*DKGRound2Package) Descriptor() ([]byte, []int) {
	return file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescGZIP(), []int{3}
}

func (x *DKGRound2Package) GetSigningShare() *SigningShare {
	if x != nil {
		return x.SigningShare
	}
	return nil
}

// Represents a commitment to a nonce value.
type NonceCommitment struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	// These bytes should be a valid group element.
	Element []byte `protobuf:"bytes,1,opt,name=element,proto3" json:"element,omitempty"`
}

func (x *NonceCommitment) Reset() {
	*x = NonceCommitment{}
	if protoimpl.UnsafeEnabled {
		mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[4]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *NonceCommitment) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*NonceCommitment) ProtoMessage() {}

func (x *NonceCommitment) ProtoReflect() protoreflect.Message {
	mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[4]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use NonceCommitment.ProtoReflect.Descriptor instead.
func (*NonceCommitment) Descriptor() ([]byte, []int) {
	return file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescGZIP(), []int{4}
}

func (x *NonceCommitment) GetElement() []byte {
	if x != nil {
		return x.Element
	}
	return nil
}

// Represents the commitments to nonces needed for signing.
type SigningCommitments struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	// One nonce to hide them.
	Hiding *NonceCommitment `protobuf:"bytes,1,opt,name=hiding,proto3" json:"hiding,omitempty"`
	// Another to bind them.
	Binding *NonceCommitment `protobuf:"bytes,2,opt,name=binding,proto3" json:"binding,omitempty"`
}

func (x *SigningCommitments) Reset() {
	*x = SigningCommitments{}
	if protoimpl.UnsafeEnabled {
		mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[5]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *SigningCommitments) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*SigningCommitments) ProtoMessage() {}

func (x *SigningCommitments) ProtoReflect() protoreflect.Message {
	mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[5]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use SigningCommitments.ProtoReflect.Descriptor instead.
func (*SigningCommitments) Descriptor() ([]byte, []int) {
	return file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescGZIP(), []int{5}
}

func (x *SigningCommitments) GetHiding() *NonceCommitment {
	if x != nil {
		return x.Hiding
	}
	return nil
}

func (x *SigningCommitments) GetBinding() *NonceCommitment {
	if x != nil {
		return x.Binding
	}
	return nil
}

// A share of the final signature. These get aggregated to make the actual thing.
type SignatureShare struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	// These bytes should be a valid scalar.
	Scalar []byte `protobuf:"bytes,1,opt,name=scalar,proto3" json:"scalar,omitempty"`
}

func (x *SignatureShare) Reset() {
	*x = SignatureShare{}
	if protoimpl.UnsafeEnabled {
		mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[6]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *SignatureShare) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*SignatureShare) ProtoMessage() {}

func (x *SignatureShare) ProtoReflect() protoreflect.Message {
	mi := &file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[6]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use SignatureShare.ProtoReflect.Descriptor instead.
func (*SignatureShare) Descriptor() ([]byte, []int) {
	return file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescGZIP(), []int{6}
}

func (x *SignatureShare) GetScalar() []byte {
	if x != nil {
		return x.Scalar
	}
	return nil
}

var File_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto protoreflect.FileDescriptor

var file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDesc = []byte{
	0x0a, 0x3c, 0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2f, 0x63, 0x72, 0x79, 0x70, 0x74,
	0x6f, 0x2f, 0x64, 0x65, 0x63, 0x61, 0x66, 0x33, 0x37, 0x37, 0x5f, 0x66, 0x72, 0x6f, 0x73, 0x74,
	0x2f, 0x76, 0x31, 0x61, 0x6c, 0x70, 0x68, 0x61, 0x31, 0x2f, 0x64, 0x65, 0x63, 0x61, 0x66, 0x33,
	0x37, 0x37, 0x5f, 0x66, 0x72, 0x6f, 0x73, 0x74, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12, 0x27,
	0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2e, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x2e,
	0x64, 0x65, 0x63, 0x61, 0x66, 0x33, 0x37, 0x37, 0x5f, 0x66, 0x72, 0x6f, 0x73, 0x74, 0x2e, 0x76,
	0x31, 0x61, 0x6c, 0x70, 0x68, 0x61, 0x31, 0x22, 0x3f, 0x0a, 0x21, 0x56, 0x65, 0x72, 0x69, 0x66,
	0x69, 0x61, 0x62, 0x6c, 0x65, 0x53, 0x65, 0x63, 0x72, 0x65, 0x74, 0x53, 0x68, 0x61, 0x72, 0x69,
	0x6e, 0x67, 0x43, 0x6f, 0x6d, 0x6d, 0x69, 0x74, 0x6d, 0x65, 0x6e, 0x74, 0x12, 0x1a, 0x0a, 0x08,
	0x65, 0x6c, 0x65, 0x6d, 0x65, 0x6e, 0x74, 0x73, 0x18, 0x01, 0x20, 0x03, 0x28, 0x0c, 0x52, 0x08,
	0x65, 0x6c, 0x65, 0x6d, 0x65, 0x6e, 0x74, 0x73, 0x22, 0xac, 0x01, 0x0a, 0x10, 0x44, 0x4b, 0x47,
	0x52, 0x6f, 0x75, 0x6e, 0x64, 0x31, 0x50, 0x61, 0x63, 0x6b, 0x61, 0x67, 0x65, 0x12, 0x6a, 0x0a,
	0x0a, 0x63, 0x6f, 0x6d, 0x6d, 0x69, 0x74, 0x6d, 0x65, 0x6e, 0x74, 0x18, 0x01, 0x20, 0x01, 0x28,
	0x0b, 0x32, 0x4a, 0x2e, 0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2e, 0x63, 0x72, 0x79,
	0x70, 0x74, 0x6f, 0x2e, 0x64, 0x65, 0x63, 0x61, 0x66, 0x33, 0x37, 0x37, 0x5f, 0x66, 0x72, 0x6f,
	0x73, 0x74, 0x2e, 0x76, 0x31, 0x61, 0x6c, 0x70, 0x68, 0x61, 0x31, 0x2e, 0x56, 0x65, 0x72, 0x69,
	0x66, 0x69, 0x61, 0x62, 0x6c, 0x65, 0x53, 0x65, 0x63, 0x72, 0x65, 0x74, 0x53, 0x68, 0x61, 0x72,
	0x69, 0x6e, 0x67, 0x43, 0x6f, 0x6d, 0x6d, 0x69, 0x74, 0x6d, 0x65, 0x6e, 0x74, 0x52, 0x0a, 0x63,
	0x6f, 0x6d, 0x6d, 0x69, 0x74, 0x6d, 0x65, 0x6e, 0x74, 0x12, 0x2c, 0x0a, 0x12, 0x70, 0x72, 0x6f,
	0x6f, 0x66, 0x5f, 0x6f, 0x66, 0x5f, 0x6b, 0x6e, 0x6f, 0x77, 0x6c, 0x65, 0x64, 0x67, 0x65, 0x18,
	0x02, 0x20, 0x01, 0x28, 0x0c, 0x52, 0x10, 0x70, 0x72, 0x6f, 0x6f, 0x66, 0x4f, 0x66, 0x4b, 0x6e,
	0x6f, 0x77, 0x6c, 0x65, 0x64, 0x67, 0x65, 0x22, 0x26, 0x0a, 0x0c, 0x53, 0x69, 0x67, 0x6e, 0x69,
	0x6e, 0x67, 0x53, 0x68, 0x61, 0x72, 0x65, 0x12, 0x16, 0x0a, 0x06, 0x73, 0x63, 0x61, 0x6c, 0x61,
	0x72, 0x18, 0x01, 0x20, 0x01, 0x28, 0x0c, 0x52, 0x06, 0x73, 0x63, 0x61, 0x6c, 0x61, 0x72, 0x22,
	0x6e, 0x0a, 0x10, 0x44, 0x4b, 0x47, 0x52, 0x6f, 0x75, 0x6e, 0x64, 0x32, 0x50, 0x61, 0x63, 0x6b,
	0x61, 0x67, 0x65, 0x12, 0x5a, 0x0a, 0x0d, 0x73, 0x69, 0x67, 0x6e, 0x69, 0x6e, 0x67, 0x5f, 0x73,
	0x68, 0x61, 0x72, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x0b, 0x32, 0x35, 0x2e, 0x70, 0x65, 0x6e,
	0x75, 0x6d, 0x62, 0x72, 0x61, 0x2e, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x2e, 0x64, 0x65, 0x63,
	0x61, 0x66, 0x33, 0x37, 0x37, 0x5f, 0x66, 0x72, 0x6f, 0x73, 0x74, 0x2e, 0x76, 0x31, 0x61, 0x6c,
	0x70, 0x68, 0x61, 0x31, 0x2e, 0x53, 0x69, 0x67, 0x6e, 0x69, 0x6e, 0x67, 0x53, 0x68, 0x61, 0x72,
	0x65, 0x52, 0x0c, 0x73, 0x69, 0x67, 0x6e, 0x69, 0x6e, 0x67, 0x53, 0x68, 0x61, 0x72, 0x65, 0x22,
	0x2b, 0x0a, 0x0f, 0x4e, 0x6f, 0x6e, 0x63, 0x65, 0x43, 0x6f, 0x6d, 0x6d, 0x69, 0x74, 0x6d, 0x65,
	0x6e, 0x74, 0x12, 0x18, 0x0a, 0x07, 0x65, 0x6c, 0x65, 0x6d, 0x65, 0x6e, 0x74, 0x18, 0x01, 0x20,
	0x01, 0x28, 0x0c, 0x52, 0x07, 0x65, 0x6c, 0x65, 0x6d, 0x65, 0x6e, 0x74, 0x22, 0xba, 0x01, 0x0a,
	0x12, 0x53, 0x69, 0x67, 0x6e, 0x69, 0x6e, 0x67, 0x43, 0x6f, 0x6d, 0x6d, 0x69, 0x74, 0x6d, 0x65,
	0x6e, 0x74, 0x73, 0x12, 0x50, 0x0a, 0x06, 0x68, 0x69, 0x64, 0x69, 0x6e, 0x67, 0x18, 0x01, 0x20,
	0x01, 0x28, 0x0b, 0x32, 0x38, 0x2e, 0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2e, 0x63,
	0x72, 0x79, 0x70, 0x74, 0x6f, 0x2e, 0x64, 0x65, 0x63, 0x61, 0x66, 0x33, 0x37, 0x37, 0x5f, 0x66,
	0x72, 0x6f, 0x73, 0x74, 0x2e, 0x76, 0x31, 0x61, 0x6c, 0x70, 0x68, 0x61, 0x31, 0x2e, 0x4e, 0x6f,
	0x6e, 0x63, 0x65, 0x43, 0x6f, 0x6d, 0x6d, 0x69, 0x74, 0x6d, 0x65, 0x6e, 0x74, 0x52, 0x06, 0x68,
	0x69, 0x64, 0x69, 0x6e, 0x67, 0x12, 0x52, 0x0a, 0x07, 0x62, 0x69, 0x6e, 0x64, 0x69, 0x6e, 0x67,
	0x18, 0x02, 0x20, 0x01, 0x28, 0x0b, 0x32, 0x38, 0x2e, 0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72,
	0x61, 0x2e, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x2e, 0x64, 0x65, 0x63, 0x61, 0x66, 0x33, 0x37,
	0x37, 0x5f, 0x66, 0x72, 0x6f, 0x73, 0x74, 0x2e, 0x76, 0x31, 0x61, 0x6c, 0x70, 0x68, 0x61, 0x31,
	0x2e, 0x4e, 0x6f, 0x6e, 0x63, 0x65, 0x43, 0x6f, 0x6d, 0x6d, 0x69, 0x74, 0x6d, 0x65, 0x6e, 0x74,
	0x52, 0x07, 0x62, 0x69, 0x6e, 0x64, 0x69, 0x6e, 0x67, 0x22, 0x28, 0x0a, 0x0e, 0x53, 0x69, 0x67,
	0x6e, 0x61, 0x74, 0x75, 0x72, 0x65, 0x53, 0x68, 0x61, 0x72, 0x65, 0x12, 0x16, 0x0a, 0x06, 0x73,
	0x63, 0x61, 0x6c, 0x61, 0x72, 0x18, 0x01, 0x20, 0x01, 0x28, 0x0c, 0x52, 0x06, 0x73, 0x63, 0x61,
	0x6c, 0x61, 0x72, 0x42, 0xeb, 0x02, 0x0a, 0x2b, 0x63, 0x6f, 0x6d, 0x2e, 0x70, 0x65, 0x6e, 0x75,
	0x6d, 0x62, 0x72, 0x61, 0x2e, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x2e, 0x64, 0x65, 0x63, 0x61,
	0x66, 0x33, 0x37, 0x37, 0x5f, 0x66, 0x72, 0x6f, 0x73, 0x74, 0x2e, 0x76, 0x31, 0x61, 0x6c, 0x70,
	0x68, 0x61, 0x31, 0x42, 0x12, 0x44, 0x65, 0x63, 0x61, 0x66, 0x33, 0x37, 0x37, 0x46, 0x72, 0x6f,
	0x73, 0x74, 0x50, 0x72, 0x6f, 0x74, 0x6f, 0x50, 0x01, 0x5a, 0x6d, 0x67, 0x69, 0x74, 0x68, 0x75,
	0x62, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2d, 0x7a,
	0x6f, 0x6e, 0x65, 0x2f, 0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2f, 0x70, 0x72, 0x6f,
	0x74, 0x6f, 0x2f, 0x67, 0x6f, 0x2f, 0x67, 0x65, 0x6e, 0x2f, 0x70, 0x65, 0x6e, 0x75, 0x6d, 0x62,
	0x72, 0x61, 0x2f, 0x63, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x2f, 0x64, 0x65, 0x63, 0x61, 0x66, 0x33,
	0x37, 0x37, 0x5f, 0x66, 0x72, 0x6f, 0x73, 0x74, 0x2f, 0x76, 0x31, 0x61, 0x6c, 0x70, 0x68, 0x61,
	0x31, 0x3b, 0x64, 0x65, 0x63, 0x61, 0x66, 0x33, 0x37, 0x37, 0x5f, 0x66, 0x72, 0x6f, 0x73, 0x74,
	0x76, 0x31, 0x61, 0x6c, 0x70, 0x68, 0x61, 0x31, 0xa2, 0x02, 0x03, 0x50, 0x43, 0x44, 0xaa, 0x02,
	0x26, 0x50, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x2e, 0x43, 0x72, 0x79, 0x70, 0x74, 0x6f,
	0x2e, 0x44, 0x65, 0x63, 0x61, 0x66, 0x33, 0x37, 0x37, 0x46, 0x72, 0x6f, 0x73, 0x74, 0x2e, 0x56,
	0x31, 0x61, 0x6c, 0x70, 0x68, 0x61, 0x31, 0xca, 0x02, 0x26, 0x50, 0x65, 0x6e, 0x75, 0x6d, 0x62,
	0x72, 0x61, 0x5c, 0x43, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x5c, 0x44, 0x65, 0x63, 0x61, 0x66, 0x33,
	0x37, 0x37, 0x46, 0x72, 0x6f, 0x73, 0x74, 0x5c, 0x56, 0x31, 0x61, 0x6c, 0x70, 0x68, 0x61, 0x31,
	0xe2, 0x02, 0x32, 0x50, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61, 0x5c, 0x43, 0x72, 0x79, 0x70,
	0x74, 0x6f, 0x5c, 0x44, 0x65, 0x63, 0x61, 0x66, 0x33, 0x37, 0x37, 0x46, 0x72, 0x6f, 0x73, 0x74,
	0x5c, 0x56, 0x31, 0x61, 0x6c, 0x70, 0x68, 0x61, 0x31, 0x5c, 0x47, 0x50, 0x42, 0x4d, 0x65, 0x74,
	0x61, 0x64, 0x61, 0x74, 0x61, 0xea, 0x02, 0x29, 0x50, 0x65, 0x6e, 0x75, 0x6d, 0x62, 0x72, 0x61,
	0x3a, 0x3a, 0x43, 0x72, 0x79, 0x70, 0x74, 0x6f, 0x3a, 0x3a, 0x44, 0x65, 0x63, 0x61, 0x66, 0x33,
	0x37, 0x37, 0x46, 0x72, 0x6f, 0x73, 0x74, 0x3a, 0x3a, 0x56, 0x31, 0x61, 0x6c, 0x70, 0x68, 0x61,
	0x31, 0x62, 0x06, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x33,
}

var (
	file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescOnce sync.Once
	file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescData = file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDesc
)

func file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescGZIP() []byte {
	file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescOnce.Do(func() {
		file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescData = protoimpl.X.CompressGZIP(file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescData)
	})
	return file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDescData
}

var file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes = make([]protoimpl.MessageInfo, 7)
var file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_goTypes = []interface{}{
	(*VerifiableSecretSharingCommitment)(nil), // 0: penumbra.crypto.decaf377_frost.v1alpha1.VerifiableSecretSharingCommitment
	(*DKGRound1Package)(nil),                  // 1: penumbra.crypto.decaf377_frost.v1alpha1.DKGRound1Package
	(*SigningShare)(nil),                      // 2: penumbra.crypto.decaf377_frost.v1alpha1.SigningShare
	(*DKGRound2Package)(nil),                  // 3: penumbra.crypto.decaf377_frost.v1alpha1.DKGRound2Package
	(*NonceCommitment)(nil),                   // 4: penumbra.crypto.decaf377_frost.v1alpha1.NonceCommitment
	(*SigningCommitments)(nil),                // 5: penumbra.crypto.decaf377_frost.v1alpha1.SigningCommitments
	(*SignatureShare)(nil),                    // 6: penumbra.crypto.decaf377_frost.v1alpha1.SignatureShare
}
var file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_depIdxs = []int32{
	0, // 0: penumbra.crypto.decaf377_frost.v1alpha1.DKGRound1Package.commitment:type_name -> penumbra.crypto.decaf377_frost.v1alpha1.VerifiableSecretSharingCommitment
	2, // 1: penumbra.crypto.decaf377_frost.v1alpha1.DKGRound2Package.signing_share:type_name -> penumbra.crypto.decaf377_frost.v1alpha1.SigningShare
	4, // 2: penumbra.crypto.decaf377_frost.v1alpha1.SigningCommitments.hiding:type_name -> penumbra.crypto.decaf377_frost.v1alpha1.NonceCommitment
	4, // 3: penumbra.crypto.decaf377_frost.v1alpha1.SigningCommitments.binding:type_name -> penumbra.crypto.decaf377_frost.v1alpha1.NonceCommitment
	4, // [4:4] is the sub-list for method output_type
	4, // [4:4] is the sub-list for method input_type
	4, // [4:4] is the sub-list for extension type_name
	4, // [4:4] is the sub-list for extension extendee
	0, // [0:4] is the sub-list for field type_name
}

func init() { file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_init() }
func file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_init() {
	if File_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto != nil {
		return
	}
	if !protoimpl.UnsafeEnabled {
		file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[0].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*VerifiableSecretSharingCommitment); i {
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
		file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[1].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*DKGRound1Package); i {
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
		file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[2].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*SigningShare); i {
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
		file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[3].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*DKGRound2Package); i {
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
		file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[4].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*NonceCommitment); i {
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
		file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[5].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*SigningCommitments); i {
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
		file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes[6].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*SignatureShare); i {
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
			RawDescriptor: file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDesc,
			NumEnums:      0,
			NumMessages:   7,
			NumExtensions: 0,
			NumServices:   0,
		},
		GoTypes:           file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_goTypes,
		DependencyIndexes: file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_depIdxs,
		MessageInfos:      file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_msgTypes,
	}.Build()
	File_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto = out.File
	file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_rawDesc = nil
	file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_goTypes = nil
	file_penumbra_crypto_decaf377_frost_v1alpha1_decaf377_frost_proto_depIdxs = nil
}
