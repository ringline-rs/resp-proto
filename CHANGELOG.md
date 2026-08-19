# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `DEFAULT_MAX_LINE_LEN` (64 KiB, matching Redis's `PROTO_INLINE_MAX_SIZE`) and a
  `max_line_len` option on `ParseOptions`.

### Fixed

- RESP line framing no longer stalls on a bare `\r`. Both scanners -- `find_crlf`
  in the value parser and `Cursor::read_line` in the command parser -- inspected
  only the first `\r` in the buffer and gave up if it was not followed by `\n`.
  A complete, CRLF-terminated line containing a stray `\r` therefore reported
  `Incomplete` forever, and since neither path is bounded, more data could never
  help: the scan restarted at the same leading `\r` every time. Reachable in
  practice because Redis keys are binary-safe and error replies echo
  user-supplied arguments. Both now scan for a real CRLF.
- RESP line framing is now bounded. Both parsers reported `Incomplete` for an
  unterminated line no matter how large the buffer grew, so a peer that never
  sent a newline could make a caller buffer without limit. Both now return
  `Protocol("line too long")` past `max_line_len`. The bound applies to protocol
  lines only; bulk payload remains bounded separately by `max_bulk_string_len`.

## [0.0.1] - 2026-02-21

### Added

- Initial release extracted from ringline workspace
- RESP2 protocol parsing and encoding
- Optional RESP3 support via `resp3` feature flag
- Streaming parser for incremental decoding
- Cluster slot and shard response parsing
