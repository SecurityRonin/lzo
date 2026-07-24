# lzo

**The safe, `no_std`, zero-dependency pure-Rust LZO1X decompressor — hardened and fuzzed against malicious input.**

```rust
// "hello, lzo world!" as a raw LZO1X block (from liblzo2's lzo1x_1).
let block = [34, 104,101,108,108,111,44,32,108,122,111,32,119,111,114,108,100,33, 17,0,0];
let mut out = [0u8; 17];                 // you supply the output capacity
let n = lzo::decompress_into(&block, &mut out).unwrap();
assert_eq!(&out[..n], b"hello, lzo world!");
```

**[GitHub Repository →](https://github.com/SecurityRonin/lzo)**

---

## Why this crate

LZO turns up in forensics and recovery — `lzop` archives, kernel/initramfs images, btrfs, anything built on liblzo2 — and there the bytes are often **untrusted, truncated, or deliberately malformed**. `lzo` is built decoder-first around the property that matters most there: **it cannot be made to misbehave on hostile input.**

- **Safe by construction** — `#![forbid(unsafe_code)]`, so every read is bounds-checked by the compiler. A corrupt or crafted block returns a typed `Error` — it can never read out of bounds, loop forever, or corrupt memory.
- **Fuzzed hard before release** — millions of arbitrary and mutation-fuzzed inputs through a libFuzzer target and a differential harness; it panics on none. Cross-validated byte-for-byte against the independent `rust-lzo` decoder.
- **Zero dependencies, `no_std`** — the core `decompress_into` needs no allocator and pulls in nothing to audit but this crate.
- **Apache-2.0-licensed** — drops cleanly into permissively-licensed projects.

It decodes streams from every `lzo1x` compressor variant (`lzo1x_1`, `lzo1x_1_15`, `lzo1x_999`) — they share one decompressor.

## Scope

- **Decompression only** (v1). Raw LZO1X has no container, so you supply the output capacity (mirroring C `lzo1x_decompress_safe`).
- Standard `lzo1x` streams. The kernel's bitstream-version (RLE) extension is out of scope.

## Correctness

Every release is round-trip tested against vectors produced by the reference C **liblzo2** — `lzo` decodes, liblzo2 encoded, so the two share no code and a mismatch can only mean `lzo` is wrong. Beyond the committed vectors, `lzo` has decoded 32.4 MB of real-world data compressed by all three liblzo2 variants, every block byte-exact, and its output matched the lineage-independent `rust-lzo` decoder across 3,000,000 mutation-fuzz inputs with zero divergence. Full methodology, results, and scope limits are on the [Validation](validation.md) page.

---

[Validation](validation.md) · [Privacy Policy](privacy.md) · [Terms of Service](terms.md) · [GitHub](https://github.com/SecurityRonin/lzo) · © 2026 Security Ronin Ltd.
