# phos

Light client for DATA Network (formerly Story Protocol) written in Rust, with native + browser + iOS support.

Phos converts an untrusted, third-party RPC endpoint into a safe, unmanipulable RPC for users. Phos is **not a fork**--it extends upstream [Helios](https://github.com/a16z/helios), adding DATA-specific consensus (CometBFT) and a custom EVM.

## Installing the node

### Installing with Cargo

Install the native node. Note `phos-cli` does not include browser support; to run Phos in a browser, build `node-wasm` from source.

```bash
cargo install phos-cli --locked
```

### Building from source

Install common dependencies.

```bash
sudo apt-get install -y build-essential curl git jq pkg-config libssl-dev

# install Rust (if necessary)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# open a new terminal or run
source "$HOME/.cargo/env"

# clone the repository
git clone https://github.com/parmenides-xyz/phos.git
cd phos

# install Phos
cargo install --path cli --locked
```

### Building node-wasm

Install Node.js 18 or newer and npm, then run:

```bash
rustup target add wasm32-unknown-unknown

cargo install wasm-pack --version 0.15.0 --locked
```

Then run:

```bash
cd node-wasm
npm ci
npm run build
```

## Running the node

### Running the node natively

For the first run, supply a trusted block height and hash:

```bash
phos node --trust-height <HEIGHT> --trust-hash <HASH>
```

To retrieve a recent candidate height and hash from DATA Network's default RPC, run:

```bash
curl -s https://story-consensus-rpc.publicnode.com/status \
  | jq -r '.result.sync_info | "height: \(.latest_block_height)\nhash: \(.latest_block_hash)"'
```

For subsequent runs:

```bash
phos node
```

View all configuration options:

```bash
phos node --help
```

### Running the node in a browser

From the repository root:

```bash
cd node-wasm/example
npm ci
npm run dev
```

The browser example is available at http://localhost:3000.
