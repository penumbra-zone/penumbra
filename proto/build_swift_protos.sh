#!/bin/zsh

brew install swift-protobuf
rm -rf swift-proto
mkdir swift-proto
protoc --proto_path=./proto --proto_path=./ibc-go-vendor --swift_out=Mgoogle/protobuf/any.proto=github.com/cosmos/cosmos-sdk/codec/types --swift_out=swift-proto ./proto/**/*.proto