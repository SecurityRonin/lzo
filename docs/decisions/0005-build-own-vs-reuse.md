# 5. Build our own safe decoder rather than reuse an existing LZO crate

Date: 2026-07-24
Status: Accepted

## Context

Several Rust LZO decoders already exist on crates.io — `rust-lzo`, `lzo1x`,
`lzokay` — so the fleet's *Research-First* and *build-vs-reuse* disciplines
(`CLAUDE.core.md`) require justifying a fresh implementation rather than pulling
one in. Each existing crate carries a trade-off that conflicts with this crate's
purpose (a safe, zero-dependency, permissively-licensed decoder for untrusted
forensic input):

- `rust-lzo` is a **GPL-2.0** port converted from Linux's
  `lzo1x_decompress_safe` — its licence cannot combine with the fleet's
  Apache-2.0/permissive posture, and it uses `unsafe`.
- The other crates differ in licence, dependency footprint, and `unsafe` usage
  (README "Why this crate").

The fleet standard is `#![forbid(unsafe_code)]` for hostile-input parsers
(ADR-0002); the constitution's refinement of "prefer our own crates" says the
research that justifies building must establish the existing `unsafe` is
*unacceptable*, not merely present. Here the GPL licence alone is disqualifying
for reuse-as-dependency, and no existing crate offers the safe + zero-dep +
Apache-2.0 combination.

## Decision

Implement the decoder from scratch as a pure-Rust, `forbid(unsafe)`,
zero-dependency crate, filling the specific niche none of the incumbents occupy.
Reuse the strongest incumbent, `rust-lzo`, **only as a local differential
oracle** — never a dependency: it lives in `validation/lzodiff`, a standalone
`publish = false` crate that is *not* a workspace member, and `Cargo.toml`
carries `exclude = ["validation/lzodiff"]` so its GPL-2.0 dependency can never
combine with — or ship inside — the published `lzo` tarball.

## Consequences

- The crate occupies a real, previously-empty niche: the safe, zero-dependency,
  Apache-2.0 LZO1X decoder; the differentiator is memory-safety by construction,
  not licence contrast (positioning corrected in commit `53a6908`).
- The prior art still earns its keep as an independent-lineage cross-check
  (ADR-0006) without contaminating the licence or dependency graph — the GPL
  code is quarantined behind a workspace boundary *and* a packaging `exclude`.
- The fleet accepts the maintenance cost of one small, fully-owned codec in
  exchange for the safety and licensing guarantees no incumbent provides.
