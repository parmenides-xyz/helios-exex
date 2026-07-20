# DATA Network iOS demo

This example runs the DATA Network light client in a native SwiftUI application. Its Xcode layout
and node lifecycle are adapted from Lumina's `examples/ios/LuminaDemo`; its interface uses the same
layout and visual tokens as `node-wasm/example`.

## Prerequisites

- Full Xcode with the iOS SDK selected by `xcode-select`
- Rust's iOS device and simulator targets

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

## Build bindings

From the repository root:

```bash
./node-uniffi/build-ios.sh
./examples/ios/DataNetworkDemo/copy_bindings.sh
```

## Run

Open `DataNetworkDemo.xcodeproj` in Xcode, select an iPhone simulator or connected iPhone, set a
development team if Xcode requests one, and run the `DataNetworkDemo` scheme.
