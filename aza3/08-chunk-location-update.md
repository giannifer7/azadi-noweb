# Updated Chunk Location Implementation

```rust
// <[@replace chunk-location]>=
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
// $$

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

impl ChunkError {
    pub fn format(&self, files: &FileStore) -> String {
        match self {
            ChunkError::RecursionLimit { chunk, location } => 
                location.format_message(
                    files,
                    MessageLevel::Error,
                    &format!(
                        "maximum recursion depth exceeded while expanding chunk '{}'",
                        chunk
                    )
                ),
            ChunkError::RecursiveReference { chunk, location } =>
                location.format_message(
                    files,
                    MessageLevel::Error,
                    &format!("recursive reference detected in chunk '{}'", chunk)
                ),
            ChunkError::UndefinedChunk { chunk, location } =>
                location.format_message(
                    files,
                    MessageLevel::Error,
                    &format!("referenced chunk '{}' is undefined", chunk)
                ),
            ChunkError::IoError(e) => format!("Error: I/O error: {}", e),
        }
    }
}

impl std::fmt::Display for ChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkError::RecursionLimit { .. } |
            ChunkError::RecursiveReference { .. } |
            ChunkError::UndefinedChunk { .. } => 
                write!(f, "{}", self.format(&FileStore::new())),
            ChunkError::IoError(e) => write!(f, "Error: I/O error: {}", e),
        }
    }
}

impl std::error::Error for ChunkError {}

impl From<io::Error> for ChunkError {
    fn from(error: io::Error) -> Self {
        ChunkError::IoError(error)
    }
}

impl From<AzadiError> for ChunkError {
    fn from(err: AzadiError) -> Self {
        ChunkError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    }
}
// $$
```