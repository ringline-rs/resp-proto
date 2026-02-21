# resp-proto

RESP2/RESP3 protocol parser and encoder for Redis.

A standalone, zero-copy protocol implementation with no runtime dependencies.

## Features

- **RESP2** parsing and encoding (enabled by default)
- **RESP3** support via the `resp3` feature flag
- Streaming parser for incremental protocol decoding
- Cluster slot parsing (CLUSTER SLOTS / CLUSTER SHARDS)

## Usage

```toml
[dependencies]
resp-proto = "0.1"

# With RESP3 support:
resp-proto = { version = "0.1", features = ["resp3"] }
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT License](LICENSE-MIT) at your option.
