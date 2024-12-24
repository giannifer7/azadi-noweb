# Fix Expansion Chunk References

```rust
// <[@replace chunk-store-expand]>=
pub fn expand_with_depth(
    &mut self,
    chunk_name: &str,
    target_indent: &str,
    depth: usize,
    seen: &mut Vec<(String, ChunkLocation)>,
    reference_location: ChunkLocation,
) -> Result<Vec<String>, ChunkError> {
    // <[expand-depth-check]>
    // <[expand-recursion-check]>
    // <[expand-chunk-access]>
    // <[expand-content]>
}

pub fn expand(&mut self, chunk_name: &str, indent: &str) -> Result<Vec<String>, ChunkError> {
    let mut seen = Vec::new();
    let initial_location = ChunkLocation {
        file_index: 0,
        line: 0,
    };
    self.expand_with_depth(chunk_name, indent, 0, &mut seen, initial_location)
}
// $$
```