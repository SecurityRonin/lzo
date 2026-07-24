# 6. Two-lineage validation: reference vectors, differential oracle, and fuzzing

Date: 2026-07-24
Status: Accepted

## Context

A decoder validated only against fixtures the author encoded is the "LZNT1 trap"
(`CLAUDE.core.md` → *Evidence-Based Rigor*): the fixture and the code can share
the same blind spot and pass green while both are wrong. A codec that emits a
value AND can be cross-checked by an independent oracle is exactly the case the
fleet requires a Tier-1/Tier-2 oracle for — a self-authored round-trip is not
enough. `lzo` produces recovered plaintext and an independent oracle is readily
available, so the bar applies in full.

## Decision

Validate correctness and robustness along two independent lineages plus fuzzing,
documented in `docs/validation.md`:

1. **Reference round-trip vectors** — every committed compressed vector is
   produced by the canonical C **liblzo2** (`lzo1x_1` / `lzo1x_999`); `lzo` must
   decode each back to the exact original. Because `lzo` does not encode
   (ADR-0001), encoder and decoder share no code, so a mismatch can only mean
   `lzo` is wrong (`tests/roundtrip.rs`, run in CI on every push).
2. **Real-world corpus** — 32.4 MB of real files (a 2.5 MB dictionary, Mach-O
   binaries, a 6 MB photo, an already-gzipped blob, source and prose) across all
   three liblzo2 variants, every block byte-exact (one-time run, reproducible via
   `validation/`).
3. **Differential oracle** — output compared byte-for-byte against `rust-lzo`, a
   lineage-independent decoder converted from Linux's `lzo1x_decompress_safe`
   (`validation/lzodiff`): identical on all 27 real blocks and across 3,000,000
   mutation-fuzz inputs with zero output divergences and zero accept/reject
   splits (commit `a98e9c0`; oracle isolation per ADR-0005).
4. **Robustness / fuzzing** — a libFuzzer target (`fuzz/fuzz_targets/decompress.rs`)
   asserting the decoder never panics on arbitrary input, plus malformed-input
   unit tests (`tests/errors.rs`); 100% line coverage reached through the public
   API alone (CI coverage gate).

## Consequences

- Correctness rests on two code lineages that share nothing (liblzo2 as encoder,
  Linux-derived `rust-lzo` as an independent decoder) — a Tier-1/Tier-2 check, not
  a self-graded fixture.
- The differential harness stays reproducible without shipping its GPL oracle:
  `validation/lzodiff` is `publish = false` and excluded from the tarball
  (ADR-0005), so the evidence is durable while the licence stays clean.
- The panic-free posture is asserted empirically (fuzzing) and statically
  (`forbid(unsafe)` + bounds-checked reads, ADR-0002) — the paired
  "input-fuzzed" + "panic-free by construction" claim the fleet README standard
  requires, not a bare unprovable "never panics".
