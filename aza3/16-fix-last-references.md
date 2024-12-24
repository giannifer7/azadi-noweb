# Fix Last Unreferenced Chunks

```rust
// <[@replace chunk-error]>=
#[derive(Debug)]
pub enum ChunkError {
    RecursionLimit {
        chunk: String,
        location: ChunkLocation,
    },
    RecursiveReference {
        chunk: String,
        location: ChunkLocation,
    },
    UndefinedChunk {
        chunk: String,
        location: ChunkLocation,
    },
    IoError(io::Error),
}

// <[chunk-error-impls]>
// $$

// <[@replace chunk-store-struct]>=
pub struct ChunkStore {
    chunks: HashMap<String, Chunk>,
    file_chunks: Vec<String>,
    files: FileStore,
    open_re: Regex,
    slot_re: Regex,
    close_re: Regex,
}

// <[path-validation]>

impl ChunkStore {
    fn validate_chunk_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        if name.starts_with("@file") {
            let path = name.trim_start_matches("@file").trim();
            if path.is_empty() || path.contains(char::is_whitespace) {
                return false;
            }
            return match path_is_safe(path) {
                Ok(_) => true,
                Err(_) => false,
            };
        }

        !name.contains(char::is_whitespace)
    }
}
// $$
```