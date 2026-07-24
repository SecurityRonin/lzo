# 1. Decompression-only, decoder-first scope

Date: 2026-07-24
Status: Accepted

## Context

LZO1X turns up in forensics and recovery — `lzop` archives, the Linux
kernel/initramfs, btrfs, anything built on liblzo2 — and in those settings the
bytes are read, not written: the examiner already has the compressed artifact
and needs to recover the plaintext. The bytes are also frequently untrusted,
truncated, or deliberately malformed. A general-purpose LZO library would carry
both an encoder and a decoder, doubling the surface and pulling in speed/ratio
trade-offs the reading use case never exercises.

The crate ships exactly one public entry point, `decompress_into` (plus an
allocating convenience `decompress`); there is no compressor in the crate at all
(`src/lib.rs`). The only encoder in the tree is `validation/lzodiff` (a C
`liblzo2` wrapper), used solely to *produce* test vectors — never shipped
(ADR-0005, ADR-0006).

## Decision

Ship a **decoder only**, built decoder-first around hostile input. Scope is the
standard `lzo1x` bitstream shared by every liblzo2 compressor variant
(`lzo1x_1`, `lzo1x_1_15`, `lzo1x_999`) — they emit one common format, so a single
`decompress_into` decodes all of them. The kernel's bitstream-version (RLE)
extension is explicitly **out of scope** (README "Scope"; `src/lib.rs` module
docs). Compression is deferred (README marks decompression as "v1", mirroring
`ruzstd` / `bzip2-rs`).

## Consequences

- The public surface is one function and one `Error` enum — trivial to audit,
  fuzz, and reason about for the untrusted-input threat model.
- A caller needing to *produce* LZO must reach for liblzo2 or another crate;
  this crate deliberately does not compete on the write path.
- Because the decoder shares no code with any encoder, a round-trip test against
  liblzo2-produced vectors is a genuine cross-check rather than a self-consistent
  tautology (ADR-0006).
- If kernel RLE streams are needed later, they arrive as an additive, opt-in
  extension rather than silently changing the standard-stream decode.
