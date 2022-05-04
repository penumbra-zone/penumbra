#!/usr/bin/env sh

protoc --proto_path=./proto --proto_path=./ibc-go-vendor --go_opt=Mgoogle/protobuf/any.proto=github.com/cosmos/cosmos-sdk/codec/types --go_out=go-proto --go_opt=paths=source_relative ./proto/*.proto
