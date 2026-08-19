use bytes::Bytes;
use resp_proto::{ParseError, ParseOptions, Value};
use std::time::Instant;

const ITERATIONS: usize = 100_000;
const MAX_INPUT_LEN: usize = 16;

#[derive(Default, Debug)]
struct Coverage {
    success: usize,
    incomplete: usize,
    error: usize,
    trailing: usize,
}

fn next_u64(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn same_error_class(left: &ParseError, right: &ParseError) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
}

#[test]
fn deterministic_resp2_parser_comparison() {
    let started = Instant::now();
    let options = ParseOptions::new()
        .max_bulk_string_len(MAX_INPUT_LEN)
        .max_collection_elements(4)
        .max_total_items(8)
        .max_depth(4);
    let mut state = 0x7265_7370_322d_6b61;
    let mut coverage = Coverage::default();

    for iteration in 0..ITERATIONS {
        let mut owned = match iteration % 8 {
            0 => b"+OK\r\n".to_vec(),
            1 => b"-ERR test\r\n".to_vec(),
            2 => b":42\r\n".to_vec(),
            3 => b"$3\r\nfoo\r\n".to_vec(),
            4 => b"*0\r\n".to_vec(),
            5 => b"$-1\r\n".to_vec(),
            6 => b":7\r\ntrailing".to_vec(),
            _ => {
                let len = (next_u64(&mut state) as usize) % (MAX_INPUT_LEN + 1);
                let mut raw = vec![0_u8; len];
                for byte in &mut raw {
                    *byte = next_u64(&mut state) as u8;
                }
                raw
            }
        };
        owned.truncate(MAX_INPUT_LEN);
        let input = owned.as_slice();
        let snapshot = input.to_vec();
        let bytes = Bytes::copy_from_slice(input);

        let copied = Value::parse_with_options(input, &options);
        let zero_copy = Value::parse_bytes_with_options(bytes.clone(), &options);

        assert_eq!(input, snapshot, "slice parser changed its input");
        assert_eq!(bytes.as_ref(), snapshot, "Bytes parser changed its input");
        match (copied, zero_copy) {
            (Ok((left, left_consumed)), Ok((right, right_consumed))) => {
                coverage.success += 1;
                coverage.trailing += usize::from(left_consumed < input.len());
                assert!(left_consumed <= input.len());
                assert_eq!(left_consumed, right_consumed);
                assert_eq!(left, right);
            }
            (Err(left), Err(right)) => {
                if left.is_incomplete() {
                    coverage.incomplete += 1;
                } else {
                    coverage.error += 1;
                }
                assert!(same_error_class(&left, &right), "{left:?} != {right:?}");
            }
            (left, right) => panic!("parser outcome mismatch: {left:?} != {right:?}"),
        }
    }

    assert_eq!(
        coverage.success + coverage.incomplete + coverage.error,
        ITERATIONS
    );
    eprintln!(
        "RESP2_RANDOM iterations={ITERATIONS} elapsed_us={} throughput_per_s={:.0} success={} incomplete={} error={} trailing={}",
        started.elapsed().as_micros(),
        ITERATIONS as f64 / started.elapsed().as_secs_f64(),
        coverage.success,
        coverage.incomplete,
        coverage.error,
        coverage.trailing,
    );
}
