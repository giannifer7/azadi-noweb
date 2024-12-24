# Fix Chunk References

```rust
// <[@file src/noweb.rs]>=
// src/noweb.rs
// <[imports]>
// <[file-store]>
// <[message-level]>

// Basic types and error handling
#[derive(Debug, Clone)]
pub struct ChunkLocation {
    pub file_index: usize,
    pub line: usize,
}

impl ChunkLocation {
    fn format_message(&self, files: &FileStore, level: MessageLevel, msg: &str) -> String {
        let file = files.get_file(self.file_index)
            .unwrap_or("<unknown>");
        format!("{}: {} {}: {}", level, file, self.line + 1, msg)
    }
}

// Chunk error handling
// <[chunk-error]>
// <[chunk-error-impls]>

// Core chunk functionality
// <[chunk-struct]>

// File path validation helper
// <[path-validation]>

// Chunk store implementation
pub struct ChunkStore {
    chunks: HashMap<String, Chunk>,
    file_chunks: Vec<String>,
    files: FileStore,
    open_re: Regex,
    slot_re: Regex,
    close_re: Regex,
}

impl ChunkStore {
    // <[chunk-store-new]>
    // <[chunk-store-validate]>
    // <[chunk-store-read]>
    
    // Chunk expansion methods
    // <[chunk-store-expand]>
    
    // Access methods
    // <[chunk-store-access]>
    // <[chunk-store-check-unused]>
}

// <[chunk-writer]>
// <[clip-struct]>
// $$
```