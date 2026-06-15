# lzo

[![Crates.io](https://img.shields.io/crates/v/lzo.svg)](https://crates.io/crates/lzo)
[![Docs.rs](https://docs.rs/lzo/badge.svg)](https://docs.rs/lzo)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![CI](https://github.com/SecurityRonin/lzo/actions/workflows/ci.yml/badge.svg)](https://github.com/SecurityRonin/lzo/actions/workflows/ci.yml)
[![Sponsor](https://img.shields.io/badge/Sponsor-%E2%9D%A4-db61a2.svg)](https://github.com/sponsors/h4x0r)

**The safe, `no_std`, zero-dependency pure-Rust LZO1X decompressor — hardened and fuzzed against malicious input.** Decode `lzo1x` data — from `lzop` files, the Linux kernel/initramfs, btrfs, or any tool built on liblzo2 — with zero C, zero dependencies, and `#![forbid(unsafe_code)]`.

```rust
// "hello, lzo world!" as a raw LZO1X block (from liblzo2's lzo1x_1).
let block = [34, 104,101,108,108,111,44,32,108,122,111,32,119,111,114,108,100,33, 17,0,0];
let mut out = [0u8; 17];                 // you supply the output capacity
let n = lzo::decompress_into(&block, &mut out).unwrap();
assert_eq!(&out[..n], b"hello, lzo world!");
```

That's it: hand it a raw block and a big-enough buffer, get the bytes back.

## Why this crate

LZO turns up in forensics and recovery — `lzop` archives, kernel/initramfs images, btrfs, anything built on liblzo2 — and there the bytes are often **untrusted, truncated, or deliberately malformed**. `lzo` is built decoder-first around the property that matters most there: **it cannot be made to misbehave on hostile input.**

- **Safe by construction** — `#![forbid(unsafe_code)]`, so every read is bounds-checked by the compiler. A corrupt or crafted block returns a typed [`Error`] — it can never read out of bounds, loop forever, or corrupt memory.
- **Fuzzed hard before release** — millions of arbitrary and mutation-fuzzed inputs through a libFuzzer target and a differential harness; it panics on none. Cross-validated byte-for-byte against the independent `rust-lzo` decoder on real data and across 3M mutated inputs with zero divergence (see [Correctness](#correctness)).
- **Zero dependencies, `no_std`** — the core `decompress_into` needs no allocator and pulls in nothing to audit but this crate.
- **Apache-2.0-licensed** — drops cleanly into permissively-licensed projects.

It decodes streams from every `lzo1x` compressor variant (`lzo1x_1`, `lzo1x_1_15`, `lzo1x_999`) — they share one decompressor. (Other Rust LZO crates exist — [`rust-lzo`](https://crates.io/crates/rust-lzo), [`lzo1x`](https://crates.io/crates/lzo1x), [`lzokay`](https://crates.io/crates/lzokay) — with different trade-offs in licence, dependencies, and `unsafe`; `lzo`'s niche is the safe, zero-dependency decoder.)

## Scope

- **Decompression only** (v1). Like `ruzstd`/`bzip2-rs`, this is a decoder; raw LZO1X has no container, so you supply the output capacity (mirroring C `lzo1x_decompress_safe`).
- Standard `lzo1x` streams. The kernel's bitstream-version (RLE) extension is out of scope.

## Correctness

Every release is round-trip tested against vectors produced by the **reference C `liblzo2`** — `lzo` decodes, liblzo2 encoded, so the two share no code and a mismatch can only mean `lzo` is wrong. The vectors cover literals, the zero-byte length extension, M1–M4 matches, overlapping copies, and the end-of-stream marker, plus malformed-input tests asserting no panic on arbitrary bytes.

Beyond the committed vectors, `lzo` has decoded **32.4 MB of real-world data** — a 2.5 MB dictionary, real Mach-O binaries, a 6 MB photo, an already-gzipped blob, real source and prose — compressed by all three liblzo2 variants (`lzo1x_1`, `lzo1x_1_15`, `lzo1x_999`), every block byte-exact.

As a second, lineage-independent check, `lzo`'s output was compared against [`rust-lzo`](https://crates.io/crates/rust-lzo) (a separate GPL decoder converted from Linux's `lzo1x_decompress_safe`, used only as a local oracle — never a dependency): identical on all 27 real blocks, and across **3,000,000** mutation-fuzz inputs there were **zero** output divergences (903k accepted by both decoders) and zero accept/reject splits. Full methodology, results, and scope limits: **[docs/validation.md](docs/validation.md)**.

[Privacy Policy](https://securityronin.github.io/lzo/privacy/) · [Terms of Service](https://securityronin.github.io/lzo/terms/) · © 2026 Security Ronin Ltd
