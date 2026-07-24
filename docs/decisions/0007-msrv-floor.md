# 7. Low declared MSRV floor, distinct from the dev toolchain pin

Date: 2026-07-24
Status: Accepted

## Context

The fleet MSRV policy (`CLAUDE.core.md` → "Rust MSRV & Toolchain Policy";
`CLAUDE.personal.md` → fleet specifics) separates two numbers that are easy to
conflate:

- the **dev toolchain** — one pinned current stable across the whole fleet, in
  `rust-toolchain.toml` (here `channel = "1.96.0"`), used to build, fmt, and
  clippy; and
- the **declared MSRV** (`rust-version` in `Cargo.toml`) — a downstream-facing
  compatibility promise that should stay low and CI-verified for a *published
  library*, and be raised only when a genuinely newer-Rust feature is needed.

`lzo` is a published library (a linked primitive, ADR-0003), so it must keep a
low MSRV floor rather than tracking the drifting dev pin; raising a library's
declared MSRV narrows its crates.io audience and is treated as near-breaking.

## Decision

Declare `rust-version = "1.85"` in `Cargo.toml` while `rust-toolchain.toml` pins
the dev toolchain to `1.96.0` (commit `2a62c0d`). The declared floor is the
promise to downstream consumers; the pin is only what the fleet develops with.

## Consequences

- Consumers on a Rust as old as 1.85 can link `lzo`, independent of whatever
  current stable the fleet develops on.
- The two numbers move independently: bumping the fleet dev toolchain does not
  silently raise the library's promise, and the MSRV rises only by a deliberate,
  reasoned edit.

## Unrecovered rationale

Why the floor is **1.85** specifically (rather than the fleet's more common
`1.75`/`1.80` library floors, or the ~1.81 that `core::error::Error` in `no_std`
would strictly require) is not recorded in the README, `Cargo.toml` comments, or
git history. *Rationale reconstructed from structure; original intent not
recovered in available history.* The split-of-two-numbers decision above **is**
grounded (the constitution + the `Cargo.toml`/`rust-toolchain.toml` values); only
the exact choice of `1.85` is not.
