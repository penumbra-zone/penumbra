#!/bin/zsh

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
cargo build --target-dir target
for i in "aarch64-apple-ios" "x86_64-apple-darwin" "aarch64-apple-darwin" "x86_64-apple-ios" "aarch64-apple-ios-sim"
do
  echo "adding & building target $i"
  echo "rustup target add $i"
  rustup target add $i
  echo "cargo build --release --target-dir target --target $i"
  cargo build --release --target-dir target --target $i
done
rustup component add rust-src --toolchain nightly-aarch64-apple-darwin

cargo +nightly build --release -Z build-std --target x86_64-apple-ios-macabi --target-dir target
cargo +nightly build --release -Z build-std --target aarch64-apple-ios-macabi --target-dir target
lipo -create \
  target/x86_64-apple-darwin/release/libpenumbra_c_bindings.a \
  target/aarch64-apple-darwin/release/libpenumbra_c_bindings.a \
  -output libpenumbra_c_bindings_macos.a
lipo -create \
  target/x86_64-apple-ios/release/libpenumbra_c_bindings.a \
  target/aarch64-apple-ios-sim/release/libpenumbra_c_bindings.a \
  -output libpenumbra_c_bindings_iossimulator.a
lipo -create \
  target/x86_64-apple-ios-macabi/release/libpenumbra_c_bindings.a \
  target/aarch64-apple-ios-macabi/release/libpenumbra_c_bindings.a \
  -output libpenumbra_c_bindings_maccatalyst.a

xcodebuild -create-xcframework \
  -library ./libpenumbra_c_bindings_macos.a \
  -headers ./include/ \
  -library ./libpenumbra_c_bindings_iossimulator.a \
  -headers ./include/ \
  -library ./libpenumbra_c_bindings_maccatalyst.a \
  -headers ./include/ \
  -library ./target/aarch64-apple-ios/release/libpenumbra_c_bindings.a \
  -headers ./include/ \
  -output penumbra_c_bindings.xcframework

zip -r bundle.zip penumbra_c_bindings.xcframework

openssl dgst -sha256 bundle.zip