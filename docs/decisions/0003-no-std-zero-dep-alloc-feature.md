# 3. `no_std`, zero external dependencies; allocation behind the `alloc` feature

Date: 2026-07-24
Status: Accepted

## Context

`lzo` is a primitive that other fleet crates link to recover LZO-compressed
artifact bytes (kernel/initramfs images, btrfs extents, `lzop` blobs). A
primitive that pulls in an allocator or third-party crates forces every consumer
to inherit that surface and forecloses the most constrained environments (a
`no_std` embedded or kernel-adjacent context). The core decode — walking `src`
into a caller-provided `dst` — needs no heap at all: it writes into a slice the
caller owns (ADR-0004).

At the same time, the common host-side ergonomics (return a `Vec` sized to a
caller-supplied upper bound) do need an allocator, and forcing every caller to
pre-size a stack buffer would be hostile to the majority use case.

## Decision

- `#![no_std]` at the crate root (`src/lib.rs`), with **zero external
  dependencies** — the dependency tree to audit is this crate alone.
- The core `decompress_into(src, dst)` is allocation-free and available in pure
  `no_std`.
- The allocating convenience `decompress(src, max_len) -> Vec<u8>` sits behind an
  `alloc` Cargo feature (`extern crate alloc`), which is **on by default**
  (`[features] default = ["alloc"]` in `Cargo.toml`) so the zero-config host
  experience is ergonomic, while `default-features = false` yields a pure
  allocation-free `no_std` build.

## Consequences

- Any consumer — host tool or `no_std` target — links the decoder with nothing
  else to vet, honoring the fleet's "prefer our own, minimal-surface" preference
  for a primitive.
- The `alloc`-on-by-default split is the library-tier reading of the fleet's
  batteries-included rule: the *library* keeps a lean opt-out path for
  constrained third-party reuse, while the convenience the common caller wants is
  present without configuration (`CLAUDE.core.md` batteries-included exception —
  "the library's default may stay lean for third-party reuse").
- `decompress` cannot over-allocate on a hostile length field: it allocates the
  caller's `max_len` bound up front and truncates to the decoded size, so the
  cap is the caller's, not an attacker's (`src/lib.rs`).
