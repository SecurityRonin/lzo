//! Round-trip tests against authoritative vectors produced by the reference C
//! liblzo2 `lzo1x_1_compress` (see `tests/data/*.{raw,lzo}` — `<name>.raw` is the
//! original, `<name>.lzo` the compressed block). Each block must decompress to
//! exactly its original.

use std::path::Path;

const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data");

fn vector(name: &str) -> (Vec<u8>, Vec<u8>) {
    let raw = std::fs::read(Path::new(DATA).join(format!("{name}.raw"))).unwrap();
    let lzo = std::fs::read(Path::new(DATA).join(format!("{name}.lzo"))).unwrap();
    (raw, lzo)
}

fn check(name: &str) {
    let (raw, lzo) = vector(name);
    // Use the core (allocation-free) API so this builds with --no-default-features.
    let mut out = vec![0u8; raw.len()];
    let n =
        lzo::decompress_into(&lzo, &mut out).unwrap_or_else(|e| panic!("decompress {name}: {e}"));
    out.truncate(n);
    assert_eq!(out, raw, "{name} round-trip mismatch");
}

#[test]
fn empty() {
    check("empty");
}

#[test]
fn hello_literals() {
    check("hello");
}

#[test]
fn run_a_match_and_length_extension() {
    check("run_a");
}

#[test]
fn pattern_distance_matches() {
    check("pattern");
}

#[test]
fn incompressible_long_literal_runs() {
    check("incompressible");
}

#[test]
fn farmatch_m3_m4_distances() {
    check("farmatch");
}
