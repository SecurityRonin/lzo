# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2](https://github.com/SecurityRonin/lzo/compare/lzo-v0.1.1...lzo-v0.1.2) - 2026-07-24

### Documentation

- complete MkDocs site (mkdocs.yml + deploy workflow)
- use verbatim Apache-2.0 license text

## [0.1.1] — 2026-06-07

No API or behaviour changes — `0.1.1` is documentation, validation, and metadata.

### Added

- **Differential validation against `rust-lzo`** — an independent decoder
  converted from Linux's `lzo1x_decompress_safe`, used as a local oracle (never a
  dependency). Identical output on the full real corpus and across 3,000,000
  mutation-fuzz inputs with zero divergence and no panics. Harness:
  `validation/lzodiff`.
- **`docs/validation.md`** documenting the full methodology — reference-encoder
  round-trips, a 32.4 MB real-world corpus, the differential check, fuzzing, and
  coverage — with scope limits stated plainly. Real-content regression vectors
  (`lzo1x_999`) added to the committed test set.

### Changed

- Documentation and positioning now lead with the real differentiator — safety
  against malicious, crafted, or corrupted input (`#![forbid(unsafe_code)]`,
  fuzz-hardened, typed errors, cross-validated) — rather than licence contrast.

## [0.1.0] — 2026-06-06

### Added

- Initial release: a safe, `no_std`, zero-dependency pure-Rust **LZO1X
  decompressor**. `decompress_into` (core, allocation-free) and an allocating
  `decompress` behind the `alloc` feature. Decodes `lzo1x_1` / `lzo1x_1_15` /
  `lzo1x_999` streams (they share one decompressor), `#![forbid(unsafe_code)]`,
  validated round-trip against reference liblzo2 vectors, and fuzz-hardened.
