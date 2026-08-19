//! Bounded proofs for the pure RESP2 framing seam.
//!
//! `find_crlf` controls every line/header boundary in the RESP2 value parser.
//! The proof covers all `sum(256^n, n = 0..8)` byte strings, with unwind ten
//! covering both scans over the maximum eight-byte input. Full parser behavior
//! and slice/zero-copy agreement are bridged by `tests/resp2_random.rs`.

use crate::value::find_crlf;

const RAW_BYTES: usize = 8;

#[kani::proof]
#[kani::unwind(10)]
fn crlf_classifier_returns_the_first_complete_delimiter() {
    let input: [u8; RAW_BYTES] = kani::any();
    let len: usize = kani::any();
    kani::assume(len <= RAW_BYTES);
    let snapshot = input;
    let data = &input[..len];

    match find_crlf(data) {
        Some(position) => {
            assert!(position + 1 < len);
            assert_eq!(data[position], b'\r');
            assert_eq!(data[position + 1], b'\n');
            for prior in 0..position {
                assert!(data[prior] != b'\r' || data[prior + 1] != b'\n');
            }
        }
        None => {
            for position in 0..len.saturating_sub(1) {
                assert!(data[position] != b'\r' || data[position + 1] != b'\n');
            }
        }
    }
    assert_eq!(input, snapshot);
}
