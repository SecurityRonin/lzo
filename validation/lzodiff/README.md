# lzodiff — differential validation oracle (local only, NOT shipped)

Cross-checks this crate's decoder against the independent
[`rust-lzo`](https://crates.io/crates/rust-lzo) (a GPL-2.0 pure-Rust port of
Linux's `lzo1x_decompress_safe`) on (A) the real liblzo2 corpus and (B) mutation
fuzz of real blocks.

**This is a standalone `publish = false` crate.** It is deliberately *not* a
member of the `lzo` workspace and is never built by `cargo test` or `cargo
publish`, so its GPL-2.0 dependency never combines with — or ships inside — the
MIT-licensed `lzo` crate. Build it only to reproduce the validation:

```sh
RW=/dir/of/<name>.raw+<name>.<algo>.lzo/pairs cargo run --release
```

See `../../docs/validation.md` § "Differential validation" for results.
