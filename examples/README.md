# Running on iOS

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
./examples/ios/copy_bindings.sh
```

## Run

Open `examples/ios/DataNetworkDemo.xcodeproj` in Xcode, select an iPhone simulator or connected iPhone, and run the `DataNetworkDemo` scheme.
