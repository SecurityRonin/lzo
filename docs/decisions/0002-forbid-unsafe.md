# 2. `#![forbid(unsafe_code)]` — safe by construction

Date: 2026-07-24
Status: Accepted

## Context

An LZO1X decoder is a parser of attacker-controllable input: back-reference
distances, run lengths, and the zero-byte length extension all come straight
from the compressed block, and a corrupt or crafted block must never read out of
bounds, loop forever, or corrupt memory. The fleet standard for such parsers is
the *Paranoid Gatekeeper* posture (`ronin-issen/CLAUDE.md` → "Security &
Robustness Standard"): every read bounds-checked, no panic on hostile input.

The constitution's `unsafe` law (`CLAUDE.core.md` → "`unsafe` Is an Avoidable
Cost-Benefit Exception") treats `forbid(unsafe)` as both the default and the
goal, downgrading to `deny` + a bounded per-site `#[allow]` only when a real
benefit (e.g. an `mmap` fast path, as in `ewf`/`memory-forensic`) justifies
surrendering the compiler-proved memory-safety guarantee. This decoder has no
such benefit: it walks two in-memory slices (`src`, `dst`) with plain indexed
reads; there is no `mmap`, no C FFI, no zero-copy trick that needs `unsafe`.

## Decision

Set `#![forbid(unsafe_code)]` in `src/lib.rs` and `unsafe_code = "forbid"` under
`[lints.rust]` in `Cargo.toml`. `forbid` (not `deny`) is chosen precisely because
there is no site that needs an override — `forbid` cannot be locally bypassed, so
it is a *provable*, badge-able "zero places a crafted input can corrupt memory."
Bounds checking is delegated to the compiler: every input read goes through
`rd` / `rd_le16` / `length_ext`, which return `Error::InputOverrun` past the end
of the slice; every output write is capacity-checked before the copy
(`copy_literals`, `copy_match`).

## Consequences

- Memory-safety on hostile input holds by construction, not by discipline or
  review — the strongest guarantee this crate can offer, and its headline
  differentiator against the existing `unsafe`-using LZO crates (ADR-0005).
- The crate can never adopt an `mmap` or FFI fast path without first downgrading
  to `deny` + a justified bounded allow; that is a deliberate, high bar.
- Overlapping-copy correctness (`distance < length`) is implemented with an
  indexed byte-by-byte loop rather than `copy_within`, staying inside safe Rust
  while preserving LZ77 repeat semantics (`copy_match` in `src/lib.rs`).
- `safe-read` is *not* pulled in: it is a fixed-width-integer reader for
  container/filesystem parsers, whereas this codec's reads are single bytes and
  one LE `u16`, already bounds-checked by the local `rd`/`rd_le16` helpers under
  `forbid(unsafe)`. Adding the dependency would violate the zero-dependency goal
  (ADR-0003) for no safety gain.
