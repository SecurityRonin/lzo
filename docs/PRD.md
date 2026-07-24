# lzo — Purpose & Scope

*Library tier. `lzo` is a linked primitive — a pure-computation codec — not a
product an examiner runs, so this is a concise Purpose & Scope rather than a full
product-requirements doc (per the fleet PRD & ADR Standard; unified filename
`docs/PRD.md`, ADR-0003). Every claim below is grounded in a same-session read of
`src/lib.rs`, `Cargo.toml`, `README.md`, and `docs/validation.md` (2026-07-24).
Load-bearing decisions live as ADRs under [`docs/decisions/`](decisions/).*

## What it is

`lzo` is a **safe, `no_std`, zero-dependency, pure-Rust LZO1X decompressor**. It
decodes a raw `lzo1x` block — as produced by liblzo2 / `lzop`, and as found in
the Linux kernel/initramfs, btrfs, and any tool built on liblzo2 — back to the
original bytes, with `#![forbid(unsafe_code)]` and no C. The whole public surface
is one function plus its error type:

- `decompress_into(src, dst) -> Result<usize, Error>` — allocation-free core; the
  caller supplies the output buffer and its capacity (ADR-0004).
- `decompress(src, max_len) -> Result<Vec<u8>, Error>` — allocating convenience
  behind the default-on `alloc` feature (ADR-0003).
- `Error` — a typed enum (`InputOverrun`, `OutputOverrun`, `LookbehindOverrun`,
  `InputNotConsumed`, `Malformed`); a malformed block returns one of these, never
  a panic.

One decoder covers every `lzo1x` compressor variant (`lzo1x_1`, `lzo1x_1_15`,
`lzo1x_999`) — they share a common bitstream (ADR-0001).

## Who links it

Fleet forensic and recovery tools that must recover LZO-compressed artifact bytes
without a C dependency: kernel/initramfs decode, btrfs extent decompression,
`lzop` blob recovery, and any host or `no_std`/embedded consumer that needs a
safe LZO1X decode. It is a leaf primitive — nothing in the fleet depends *through*
it, and it depends on nothing but `core` (optionally `alloc`).

## What it does (and the property that matters)

The bytes `lzo` reads are routinely **untrusted, truncated, or deliberately
malformed**, so the crate is built decoder-first around one guarantee: **it
cannot be made to misbehave on hostile input.** Memory-safety holds *by
construction* — `forbid(unsafe)` (ADR-0002), every input read bounds-checked
(`rd`/`rd_le16`/`length_ext`), every output write capacity-checked
(`copy_literals`/`copy_match`), overflow-safe length arithmetic (`saturating_*`),
and overlapping back-references copied byte-by-byte to preserve LZ77 semantics.

## Scope

**In scope**

- Decompression of standard `lzo1x` blocks (all liblzo2 compressor variants).
- `no_std`, zero-dependency operation; ergonomic allocating path under `alloc`.
- Robust, typed-error handling of corrupt / truncated / crafted input.

**Non-goals**

- **Compression / encoding.** Decoder only (v1); the only encoder in the tree is
  the C oracle used to mint test vectors, never shipped (ADR-0001, ADR-0005).
- **Container / file-format parsing.** No `lzop` magic, block framing, stored
  sizes, or multi-block streaming — that is a higher layer's concern; a raw block
  carries no output size, so the caller supplies capacity (ADR-0004).
- **The kernel bitstream-version (RLE) extension** — out of scope.
- **A runnable binary / CLI / GUI** — this is a linked library; the examiner-facing
  surface lives in the fleet tools that depend on it.

## How correctness is established

Two independent code lineages plus fuzzing (full detail in
[`docs/validation.md`](validation.md); decision in ADR-0006):

- **Reference round-trips** against vectors encoded by the canonical C liblzo2 —
  encoder and decoder share no code, so a mismatch can only mean `lzo` is wrong
  (`tests/roundtrip.rs`, CI every push).
- **Real-world corpus** — 32.4 MB of real files across all three liblzo2
  variants, every block byte-exact.
- **Differential oracle** — byte-for-byte agreement with the lineage-independent
  `rust-lzo` on the real corpus and across 3,000,000 mutation-fuzz inputs (zero
  divergences, zero accept/reject splits); the GPL oracle is quarantined in
  `validation/lzodiff` and excluded from the published tarball (ADR-0005).
- **Fuzzing + coverage** — a libFuzzer target asserting no panic on arbitrary
  input (`fuzz/`), malformed-input tests (`tests/errors.rs`), and 100% line
  coverage through the public API.
