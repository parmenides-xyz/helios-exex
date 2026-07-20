#!/usr/bin/env bash

set -euxo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-18.0}"

# create swift bindings

rm -rf ./bindings ./ios
mkdir -p ./bindings
mkdir -p ./ios
mkdir -p ./bindings/Headers

cargo build

cargo run --bin uniffi-bindgen \
  generate \
  --library ../target/debug/libhelios_exex_node_uniffi.dylib \
  --language swift \
  --out-dir ./bindings

cat \
	./bindings/helios_exex_node_uniffiFFI.modulemap > ./bindings/Headers/module.modulemap

cp ./bindings/*.h ./bindings/Headers/

rm -rf ./ios/helios_exex.xcframework

# create xcode project

cargo build -p helios-exex-node-uniffi \
  --release \
  --lib \
  --target aarch64-apple-ios \
  --target aarch64-apple-ios-sim

xcodebuild -create-xcframework \
  -library ../target/aarch64-apple-ios/release/libhelios_exex_node_uniffi.a -headers ./bindings/Headers \
  -library ../target/aarch64-apple-ios-sim/release/libhelios_exex_node_uniffi.a -headers ./bindings/Headers \
  -output "ios/helios_exex.xcframework"

cp ./bindings/*.swift ./ios/

rm -rf bindings
