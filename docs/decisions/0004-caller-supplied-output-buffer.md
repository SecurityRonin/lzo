# 4. Caller-supplied output buffer; no container framing

Date: 2026-07-24
Status: Accepted

## Context

A raw LZO1X block carries **no header and no stored output size** — it is a bare
compressed stream terminated by an end-of-stream marker (`0x11 0x00 0x00`). The
decompressed length is therefore not knowable from the block alone; the caller
must supply it (or an upper bound), exactly as the canonical C
`lzo1x_decompress_safe` requires an output buffer and its capacity. Any framing
that stored the size (as `lzop`'s file format does) lives *above* the raw block,
not inside it, and is the container layer's concern — not this decoder's.

Getting this wrong in a decoder for untrusted input is a memory-safety hazard: a
length taken from the stream and trusted is exactly the alloc-bomb / overrun
vector the fleet's robustness standard exists to prevent.

## Decision

The primary API is `decompress_into(src: &[u8], dst: &mut [u8]) -> Result<usize,
Error>`: the caller owns and sizes `dst`, and the function returns the number of
bytes written. Writing past `dst` is a typed `Error::OutputOverrun`, never a
panic or a reallocation. No container parsing, no size prefix, no auto-growth is
performed at this layer. The allocating `decompress` (ADR-0003) is a thin wrapper
that takes an explicit `max_len` cap from the caller.

## Consequences

- The output capacity is always the caller's decision, so a crafted block cannot
  drive an unbounded allocation or an out-of-bounds write — secure by default,
  enforced structurally (`copy_literals`/`copy_match` capacity checks).
- Container concerns (the `lzop` magic, block framing, stored sizes, multi-block
  streams) belong to a higher layer and are deliberately absent here, keeping the
  decoder a reusable primitive rather than a `lzop` file reader.
- Callers that genuinely do not know the size must apply their own strategy
  (grow-and-retry, or a domain upper bound); the API makes that an explicit
  choice rather than hiding an unbounded allocation.
