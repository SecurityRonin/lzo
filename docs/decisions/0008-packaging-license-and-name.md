# 8. Packaging: Apache-2.0 licence and the bare-`lzo` primitive name

Date: 2026-07-24
Status: Accepted

## Context

Two packaging choices position this crate within the fleet and on crates.io, and
both are visible in the repository and git history.

**Licence.** The crate was first published MIT (`0.1.0`) and later relicensed to
**Apache-2.0** (commit `5ccbabe`, "relicense MIT → Apache-2.0 (fleet standard)").
The fleet standardized on Apache-2.0 for its explicit patent grant
(`ronin-issen/CLAUDE.md` → README standard: "the fleet standardized on
**Apache-2.0** for its explicit patent grant — migrate any residual MIT repos").
(Some in-tree comments — `Cargo.toml`'s `exclude` note and
`validation/lzodiff/Cargo.toml` — still read "MIT-licensed tarball"; these are
stale wording, the `[package] license` field is `Apache-2.0`.)

**Crate name.** The fleet's naming grammar (`ronin-issen/CLAUDE.md` → "Crate
naming grammar", "Crate-structure standard") mandates a `<x>-core` reader +
`<x>-forensic` analyzer split for **forensic format readers/analyzers**. `lzo` is
neither: it is a general-purpose compression primitive (crates.io categories
`compression`, `no-std`) that forensic tools *link*, not a reader that parses an
evidence artifact family into findings. The bare name `lzo` was available and is
what the ecosystem searches for.

## Decision

- License the crate **Apache-2.0** (single source of truth: the `LICENSE` file
  and the `Cargo.toml` `license` field; no `## License` prose section in the
  README, per the fleet README standard).
- Publish under the **bare `lzo`** crate name with **no `-core`/`-forensic`
  split**: the core/forensic grammar governs forensic format readers, and a
  pure-computation codec is a primitive that sits below that layer, so it takes a
  plain ecosystem-idiomatic name.

## Consequences

- The crate drops cleanly into permissively-licensed and Apache-2.0 projects and
  carries the patent grant the fleet requires.
- `use lzo::…` is the natural import for a compression primitive; no consumer has
  to learn a forensic-suite naming convention to depend on a codec.
- The stale "MIT" wording in two comments should be corrected in a later
  minimal-diff pass; it does not affect the actual licence, which is Apache-2.0.
