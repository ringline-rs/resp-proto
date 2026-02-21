# CLAUDE.md

## Build & Test Commands

```bash
# Build
cargo build
cargo build --all-features

# Lint
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings

# Test
cargo test --all
cargo test --all --all-features
cargo test --all --release

# Docs
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

## Architecture

Pure protocol parser/encoder for RESP2/RESP3. No runtime dependencies.

### Features

- `resp3` — enables RESP3 types (Double, BigNumber, etc.) via `ryu` dependency

### Key Types

- `Value` — parsed RESP value (SimpleString, Error, Integer, BulkString, Array, etc.)
- `Request` — client request (command + arguments)
- `Command` — parsed Redis command enum
- `StreamingParser` — incremental protocol decoder
- `ClusterSlots` / `ClusterShards` — cluster topology parsing
