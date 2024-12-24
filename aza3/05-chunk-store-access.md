# ChunkStore Access Methods

```rust
// <[chunk-store-access]>=
pub fn get_chunk_content(&mut self, chunk_name: &str) -> Result<Vec<String>, ChunkError> {
    self.expand(chunk_name, "")
}

pub fn get_file_chunks(&self) -> &[String] {
    &self.file_chunks
}

pub fn reset(&mut self) {
    self.chunks.clear();
    self.file_chunks.clear();
}

pub fn has_chunk(&self, name: &str) -> bool {
    self.chunks.contains_key(name)
}
// $$

// <[chunk-store-check-unused]>=
pub fn check_unused_chunks(&self) -> Vec<String> {
    let mut warnings = Vec::new();
    for (name, chunk) in &self.chunks {
        if !name.starts_with("@file") && chunk.reference_count == 0 {
            warnings.push(chunk.location.format_message(
                MessageLevel::Warning,
                &format!("chunk '{}' is defined but never referenced", name),
            ));
        }
    }
    warnings.sort();
    warnings
}
// $$
```