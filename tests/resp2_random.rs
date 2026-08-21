use bytes::Bytes;
use resp_proto::{ParseOptions, Value};
use std::time::Instant;

const ITERATIONS: usize = 100_000;
const MAX_INPUT_LEN: usize = 24;

#[derive(Default, Debug)]
struct Coverage {
    success: usize,
    incomplete: usize,
    error: usize,
    trailing: usize,
    roundtrip: usize,
    random: usize,
}

fn next_u64(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn input_for(iteration: usize, state: &mut u64) -> (Vec<u8>, bool) {
    let fixture = match iteration % 16 {
        0 => Some(&b"+OK\r\n"[..]),
        1 => Some(&b"-ERR test\r\n"[..]),
        2 => Some(&b":42\r\n"[..]),
        3 => Some(&b"$3\r\nfoo\r\n"[..]),
        4 => Some(&b"*0\r\n"[..]),
        5 => Some(&b"$-1\r\n"[..]),
        6 => Some(&b"*2\r\n:1\r\n:2\r\n"[..]),
        7 => Some(&b"*2\r\n*1\r\n+OK\r\n$3\r\nfoo\r\n"[..]),
        8 => Some(&b":7\r\ntrailing"[..]),
        9 => Some(&b"+OK\r"[..]),
        10 => Some(&b"$3\r\nfo"[..]),
        11 => Some(&b"*2\r\n:1\r\n"[..]),
        12 => Some(&b"$3\r\nfooXX"[..]),
        13 => Some(&b":not-int\r\n"[..]),
        _ => None,
    };
    if let Some(fixture) = fixture {
        return (fixture.to_vec(), false);
    }

    let len = (next_u64(state) as usize) % (MAX_INPUT_LEN + 1);
    let mut raw = vec![0_u8; len];
    for byte in &mut raw {
        *byte = next_u64(state) as u8;
    }
    (raw, true)
}

#[test]
fn bare_cr_before_crlf_is_line_content() {
    let expected = Value::simple_string(b"bad\rX");
    assert_eq!(Value::parse(b"+bad\rX\r\n").unwrap(), (expected.clone(), 8));
    assert_eq!(
        Value::parse_bytes(Bytes::from_static(b"+bad\rX\r\n")).unwrap(),
        (expected, 8)
    );
}

#[test]
fn deterministic_resp2_parser_diagnostic() {
    let started = Instant::now();
    let options = ParseOptions::new()
        .max_bulk_string_len(MAX_INPUT_LEN)
        .max_collection_elements(4)
        .max_total_items(8)
        .max_depth(4);
    let mut state = 0x7265_7370_322d_6b61;
    let mut coverage = Coverage::default();

    for iteration in 0..ITERATIONS {
        let (mut owned, random) = input_for(iteration, &mut state);
        owned.truncate(MAX_INPUT_LEN);
        coverage.random += usize::from(random);
        let input = owned.as_slice();
        let copied = Value::parse_with_options(input, &options);
        let zero_copy = Value::parse_bytes_with_options(Bytes::copy_from_slice(input), &options);

        match (copied, zero_copy) {
            (Ok((left, left_consumed)), Ok((right, right_consumed))) => {
                coverage.success += 1;
                coverage.trailing += usize::from(left_consumed < input.len());
                assert!(left_consumed <= input.len());
                assert_eq!(left_consumed, right_consumed);
                assert_eq!(left, right);

                let mut encoded = vec![0_u8; left.encoded_len()];
                let written = left.encode(&mut encoded);
                let (roundtrip, consumed) = Value::parse_with_options(&encoded, &options).unwrap();
                assert_eq!(roundtrip, left);
                assert_eq!(consumed, written);
                coverage.roundtrip += 1;
            }
            (Err(left), Err(right)) => {
                if left.is_incomplete() {
                    coverage.incomplete += 1;
                } else {
                    coverage.error += 1;
                }
                assert_eq!(left, right);
            }
            (left, right) => panic!("parser outcome mismatch: {left:?} != {right:?}"),
        }
    }

    let elapsed = started.elapsed();
    assert_eq!(
        coverage.success + coverage.incomplete + coverage.error,
        ITERATIONS
    );
    assert_eq!(coverage.roundtrip, coverage.success);
    eprintln!(
        "RESP2_DIAGNOSTIC iterations={ITERATIONS} elapsed_us={} throughput_per_s={:.0} success={} incomplete={} error={} trailing={} roundtrip={} random={}",
        elapsed.as_micros(),
        ITERATIONS as f64 / elapsed.as_secs_f64(),
        coverage.success,
        coverage.incomplete,
        coverage.error,
        coverage.trailing,
        coverage.roundtrip,
        coverage.random,
    );
}
