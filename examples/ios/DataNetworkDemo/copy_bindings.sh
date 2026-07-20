#!/usr/bin/env bash

set -euxo pipefail

cd "$(git rev-parse --show-toplevel)"

cp -rv node-uniffi/ios/helios_exex.xcframework examples/ios/DataNetworkDemo
cp -v node-uniffi/ios/*.swift examples/ios/DataNetworkDemo/DataNetworkDemo
