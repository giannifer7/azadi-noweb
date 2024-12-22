# ChunkStore Implementation

## Main Structure

The ChunkStore struct holds the core state:

```rust
// <[chunk-store-struct]>=
pub struct ChunkStore {
    chunks: HashMap<String, Chunk>,
    file_chunks: Vec<String>,
    open_re: Regex,
    slot_re: Regex,
    close_re: Regex,
}
// $$
```

## Constructor

Constructor that sets up regex patterns for chunk recognition. Note the escape handling for delimiters and comment markers:

## Chunk Name Validation

Validation for chunk names and paths:

## Reading Implementation

The core reading logic, now with support for @replace:



We'll continue with expansion and reference handling in the next chunk. Would you like me to proceed with that?
