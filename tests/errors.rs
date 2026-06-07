//! Error-path and robustness tests. Every case drives the public API on
//! malformed, truncated, or crafted input — so these double as e2e coverage of
//! the decoder's guards, and assert it never panics on hostile bytes.

use lzo::{decompress_into, Error};

// "hello, lzo world!" compressed by liblzo2's lzo1x_1, then the EOF marker.
const HELLO: &[u8] = &[
    34, 104, 101, 108, 108, 111, 44, 32, 108, 122, 111, 32, 119, 111, 114, 108, 100, 33, 0x11,
    0x00, 0x00,
];

#[test]
fn input_shorter_than_marker_is_overrun() {
    assert_eq!(
        decompress_into(&[0x11, 0x00], &mut []),
        Err(Error::InputOverrun)
    );
}

#[test]
fn output_too_small_is_overrun() {
    let mut small = [0u8; 5]; // "hello, lzo world!" needs 17
    assert_eq!(
        decompress_into(HELLO, &mut small),
        Err(Error::OutputOverrun)
    );
}

#[test]
fn match_before_output_start_is_lookbehind_overrun() {
    // 1 literal (0x12), then an M3 match (0x21) whose distance reaches before it.
    assert_eq!(
        decompress_into(
            &[0x12, b'X', 0x21, 0x08, 0x00, 0x11, 0x00, 0x00],
            &mut [0u8; 64]
        ),
        Err(Error::LookbehindOverrun)
    );
}

#[test]
fn trailing_bytes_after_eof_are_rejected() {
    assert_eq!(
        decompress_into(&[0x11, 0x00, 0x00, 0x99], &mut [0u8; 4]),
        Err(Error::InputNotConsumed)
    );
}

#[test]
fn truncated_literal_run_is_input_overrun() {
    // 0x15 = 21 > 17 -> initial literal run of 4 bytes, but only 2 follow.
    assert_eq!(
        decompress_into(&[0x15, b'a', b'b'], &mut [0u8; 16]),
        Err(Error::InputOverrun)
    );
}

#[test]
fn oversized_length_run_is_rejected() {
    // M3 (0x20) with a zero-byte length extension that never terminates before
    // the input ends -> InputOverrun (the run-length-overflow guard sits behind
    // an impractically long run, so exhaustion fires first).
    let mut block = vec![0x12, b'X', 0x20];
    block.extend(std::iter::repeat_n(0u8, 4096));
    assert_eq!(
        decompress_into(&block, &mut [0u8; 64]),
        Err(Error::InputOverrun)
    );
}

#[test]
fn match_overflowing_output_is_overrun() {
    // 1 literal 'A' (0x12), then a 2-byte M1 match (distance 1) into a 2-byte
    // buffer that has only 1 free slot -> the match copy overruns the output.
    assert_eq!(
        decompress_into(&[0x12, b'A', 0x00, 0x00], &mut [0u8; 2]),
        Err(Error::OutputOverrun)
    );
}

#[test]
fn m4_end_marker_with_wrong_length_is_malformed() {
    // 1 literal (0x12), then an M4 op (0x10) whose length extension makes the
    // length 10 while its distance is zero — i.e. it lands on the end-of-stream
    // condition but is not the canonical length-3 marker.
    assert_eq!(
        decompress_into(&[0x12, b'X', 0x10, 0x01, 0x00, 0x00], &mut [0u8; 64]),
        Err(Error::Malformed)
    );
}

#[test]
fn arbitrary_bytes_never_panic() {
    let mut out = [0u8; 4096];
    for seed in 0u32..20_000 {
        let mut s = seed.wrapping_mul(2_654_435_761).wrapping_add(1);
        let mut buf = [0u8; 24];
        for b in &mut buf {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            *b = (s >> 24) as u8;
        }
        let _ = decompress_into(&buf, &mut out);
    }
}

#[test]
fn error_messages_are_distinct_and_prefixed() {
    let all = [
        Error::InputOverrun,
        Error::OutputOverrun,
        Error::LookbehindOverrun,
        Error::InputNotConsumed,
        Error::Malformed,
    ];
    let mut msgs: Vec<String> = all.iter().map(ToString::to_string).collect();
    assert!(msgs.iter().all(|m| m.starts_with("lzo: ")));
    msgs.sort();
    msgs.dedup();
    assert_eq!(msgs.len(), 5);
    // Exercise Debug + the core::error::Error impl too.
    let _: &dyn std::error::Error = &Error::Malformed;
    assert!(format!("{:?}", Error::Malformed).contains("Malformed"));
}

#[cfg(feature = "alloc")]
#[test]
fn decompress_convenience_allocates_and_truncates() {
    assert_eq!(lzo::decompress(&[0x11, 0x00, 0x00], 0).unwrap(), b"");
    assert_eq!(lzo::decompress(HELLO, 17).unwrap(), b"hello, lzo world!");
    // max_len larger than needed: the result is truncated to the real length.
    assert_eq!(lzo::decompress(HELLO, 100).unwrap().len(), 17);
}
