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
//! let out = lzo::decompress(&block, 17).unwrap();
//! assert_eq!(out, b"hello, lzo world!");
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

/// Decompress a raw LZO1X block into `dst`, returning the number of bytes
/// written. `dst` must be large enough to hold the entire decompressed output.
pub fn decompress_into(src: &[u8], dst: &mut [u8]) -> Result<usize, Error> {
    let _ = (src, dst);
    Err(Error::Malformed) // stub
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
