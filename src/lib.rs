//! Safe, dependency-free, pure-Rust **LZO1X decompressor**.
//!
//! Decodes raw LZO1X blocks as produced by `lzo1x_1`, `lzo1x_1_15`, and
//! `lzo1x_999` (the liblzo2 / `lzop` family) — a single compressed block with
//! no container header or stored output size. The output length must be known
//! (or upper-bounded) by the caller, exactly like the C `lzo1x_decompress_safe`.
//!
//! This is the GPL-free, MIT-licensed alternative to the existing GPL Rust LZO
//! ports, with `#![forbid(unsafe_code)]` and zero dependencies.
//!
//! ```
//! // "hello, lzo world!" compressed by liblzo2's lzo1x_1.
//! let block = [34, 104,101,108,108,111,44,32,108,122,111,32,119,111,114,108,100,33, 17,0,0];
//! let mut out = [0u8; 17];
//! let n = lzo::decompress_into(&block, &mut out).unwrap();
//! assert_eq!(&out[..n], b"hello, lzo world!");
//! ```
#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::fmt;

/// An error returned while decompressing an LZO1X block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The compressed input ended before the block was fully decoded.
    InputOverrun,
    /// Decoding would write past the end of the output buffer.
    OutputOverrun,
    /// A back-reference points before the start of the output (corrupt block).
    LookbehindOverrun,
    /// The block decoded to a valid end but bytes remain in the input.
    InputNotConsumed,
    /// The block is malformed (bad instruction, run-length overflow, …).
    Malformed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::InputOverrun => "lzo: compressed input ended prematurely",
            Error::OutputOverrun => "lzo: output buffer too small",
            Error::LookbehindOverrun => "lzo: back-reference before output start",
            Error::InputNotConsumed => "lzo: trailing bytes after end-of-stream",
            Error::Malformed => "lzo: malformed compressed block",
        })
    }
}

impl core::error::Error for Error {}

/// Read one byte and advance.
fn rd(src: &[u8], ip: &mut usize) -> Result<u8, Error> {
    let b = *src.get(*ip).ok_or(Error::InputOverrun)?;
    *ip += 1;
    Ok(b)
}

/// Read a little-endian `u16` and advance.
fn rd_le16(src: &[u8], ip: &mut usize) -> Result<usize, Error> {
    let lo = rd(src, ip)? as usize;
    let hi = rd(src, ip)? as usize;
    Ok(lo | (hi << 8))
}

/// The zero-byte length extension: skip a run of `0x00` bytes (each worth 255)
/// then read the terminating non-zero byte; returns `255*zeros + terminator`
/// (the caller adds the per-command base). Guards against run-length overflow.
fn length_ext(src: &[u8], ip: &mut usize) -> Result<usize, Error> {
    const MAX_255_COUNT: usize = usize::MAX / 255 - 2;
    let mut zeros = 0usize;
    while *src.get(*ip).ok_or(Error::InputOverrun)? == 0 {
        *ip += 1;
        zeros += 1;
        if zeros > MAX_255_COUNT {
            return Err(Error::Malformed);
        }
    }
    let term = rd(src, ip)? as usize;
    Ok(zeros * 255 + term)
}

/// Copy `n` literal bytes from the input to the output.
fn copy_literals(
    src: &[u8],
    ip: &mut usize,
    dst: &mut [u8],
    op: &mut usize,
    n: usize,
) -> Result<(), Error> {
    if n > dst.len() - *op {
        return Err(Error::OutputOverrun);
    }
    if n > src.len() - *ip {
        return Err(Error::InputOverrun);
    }
    dst[*op..*op + n].copy_from_slice(&src[*ip..*ip + n]);
    *op += n;
    *ip += n;
    Ok(())
}

/// Copy a `length`-byte back-reference `distance` bytes behind the output
/// cursor — byte-by-byte, so an overlapping copy (distance < length) repeats
/// correctly, as LZ77 requires.
fn copy_match(dst: &mut [u8], op: &mut usize, distance: usize, length: usize) -> Result<(), Error> {
    if distance == 0 || distance > *op {
        return Err(Error::LookbehindOverrun);
    }
    if length > dst.len() - *op {
        return Err(Error::OutputOverrun);
    }
    // Forward byte-by-byte so an overlapping copy (distance < length) repeats
    // the just-written bytes, as LZ77 requires — `copy_within` (memmove) would
    // not. Indexed because the source and destination ranges overlap.
    let s = *op - distance;
    #[allow(clippy::needless_range_loop)]
    for i in 0..length {
        dst[*op + i] = dst[s + i];
    }
    *op += length;
    Ok(())
}

/// Decompress a raw LZO1X block into `dst`, returning the number of bytes
/// written. `dst` must be large enough to hold the entire decompressed output.
///
/// Decodes standard `lzo1x_*` streams (no kernel bitstream-version extension).
pub fn decompress_into(src: &[u8], dst: &mut [u8]) -> Result<usize, Error> {
    if src.len() < 3 {
        return Err(Error::InputOverrun);
    }
    let mut ip = 0usize;
    let mut op = 0usize;
    let mut state: usize;
    let mut t: usize;

    // First instruction: a first byte > 17 is an initial literal run of
    // `byte - 17` (1..3 of them behave like a prior op's trailing literals).
    let first = src[0];
    if first > 17 {
        ip = 1;
        t = first as usize - 17;
        copy_literals(src, &mut ip, dst, &mut op, t)?;
        state = if t < 4 { t } else { 4 };
    } else {
        state = 0; // first byte is read as an ordinary instruction below
    }

    loop {
        t = rd(src, &mut ip)? as usize;
        let (distance, length, next);
        if t < 16 {
            if state == 0 {
                // Literal run: length t+3, with the zero-byte extension (base 18).
                let len = if t == 0 {
                    18 + length_ext(src, &mut ip)?
                } else {
                    t + 3
                };
                copy_literals(src, &mut ip, dst, &mut op, len)?;
                state = 4;
                continue;
            }
            // state > 0: a short match from a near distance.
            next = t & 3;
            if state == 4 {
                distance = 1 + 2048 + (t >> 2) + ((rd(src, &mut ip)? as usize) << 2);
                length = 3;
            } else {
                distance = 1 + (t >> 2) + ((rd(src, &mut ip)? as usize) << 2);
                length = 2;
            }
        } else if t >= 64 {
            // M2: short match, 3 distance bits in `t` + one follow byte.
            next = t & 3;
            distance = 1 + ((t >> 2) & 7) + ((rd(src, &mut ip)? as usize) << 3);
            length = (t >> 5) + 1;
        } else if t >= 32 {
            // M3: length in low 5 bits (base 2 / extension base 33), le16 distance.
            length = if (t & 31) == 0 {
                33 + length_ext(src, &mut ip)?
            } else {
                (t & 31) + 2
            };
            let d = rd_le16(src, &mut ip)?;
            distance = 1 + (d >> 2);
            next = d & 3;
        } else {
            // M4 (16..=31): long match, or the end-of-stream marker.
            let hi = (t & 8) << 11;
            length = if (t & 7) == 0 {
                9 + length_ext(src, &mut ip)?
            } else {
                (t & 7) + 2
            };
            let d = rd_le16(src, &mut ip)?;
            let dist_part = d >> 2;
            next = d & 3;
            if hi == 0 && dist_part == 0 {
                // End of stream (canonical marker `0x11 0x00 0x00`).
                if length != 3 {
                    return Err(Error::Malformed);
                }
                if ip < src.len() {
                    return Err(Error::InputNotConsumed);
                }
                return Ok(op);
            }
            distance = hi + dist_part + 0x4000;
        }
        copy_match(dst, &mut op, distance, length)?;
        copy_literals(src, &mut ip, dst, &mut op, next)?;
        state = next;
    }
}

/// Decompress a raw LZO1X block, allocating an output of at most `max_len`
/// bytes. Returns the decompressed data.
#[cfg(feature = "alloc")]
pub fn decompress(src: &[u8], max_len: usize) -> Result<alloc::vec::Vec<u8>, Error> {
    let mut dst = alloc::vec![0u8; max_len];
    let n = decompress_into(src, &mut dst)?;
    dst.truncate(n);
    Ok(dst)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Canonical empty stream: the 3-byte end-of-stream marker.
    #[cfg(feature = "alloc")]
    const EOF: &[u8] = &[0x11, 0x00, 0x00];

    #[cfg(feature = "alloc")]
    #[test]
    fn empty_stream_decodes_to_nothing() {
        assert_eq!(decompress(EOF, 0).unwrap(), b"");
    }

    #[test]
    fn input_shorter_than_marker_is_overrun() {
        assert_eq!(
            decompress_into(&[0x11, 0x00], &mut []),
            Err(Error::InputOverrun)
        );
    }

    #[test]
    fn output_too_small_is_overrun() {
        // "hello, lzo world!" needs 17 bytes; give it 5.
        let block = [
            34, 104, 101, 108, 108, 111, 44, 32, 108, 122, 111, 32, 119, 111, 114, 108, 100, 33,
            0x11, 0x00, 0x00,
        ];
        let mut small = [0u8; 5];
        assert_eq!(
            decompress_into(&block, &mut small),
            Err(Error::OutputOverrun)
        );
    }

    #[test]
    fn match_before_output_start_is_lookbehind_overrun() {
        // Emit 1 literal (0x12 -> initial run of 1), then an M3 match (0x21) whose
        // distance (3) reaches before the single output byte.
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
        let mut buf = [0u8; 4];
        assert_eq!(
            decompress_into(&[0x11, 0x00, 0x00, 0x99], &mut buf),
            Err(Error::InputNotConsumed)
        );
    }

    #[test]
    fn truncated_literal_run_is_input_overrun() {
        // First byte 0x15 = 21 > 17 -> initial literal run of 4 bytes, but only 2 follow.
        assert_eq!(
            decompress_into(&[0x15, b'a', b'b'], &mut [0u8; 16]),
            Err(Error::InputOverrun)
        );
    }

    #[test]
    fn arbitrary_bytes_never_panic() {
        // No input should panic; every result is Ok or a typed Err.
        let mut out = [0u8; 256];
        for seed in 0u32..4000 {
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

    #[cfg(feature = "alloc")]
    #[test]
    fn error_messages_are_distinct() {
        use core::fmt::Write;
        let mut seen = alloc::vec::Vec::new();
        for e in [
            Error::InputOverrun,
            Error::OutputOverrun,
            Error::LookbehindOverrun,
            Error::InputNotConsumed,
            Error::Malformed,
        ] {
            let mut s = alloc::string::String::new();
            write!(s, "{e}").unwrap();
            assert!(s.starts_with("lzo: "));
            seen.push(s);
        }
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 5);
    }
}
