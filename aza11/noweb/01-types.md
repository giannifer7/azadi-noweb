# Types and Error Handling

## Location Tracking

The ChunkLocation type provides source information for error reporting:

```rust
// <[chunk-location]>=
#[derive(Debug, Clone)]
pub struct ChunkLocation {
    pub file: String,
    pub line: usize,
}
// $$
```

## Error Level Handling

We differentiate between errors and warnings:

```rust
// <[message-level]>=
#[derive(Debug, Clone, Copy)]
enum MessageLevel {
    Error,
    Warning,
}

impl std::fmt::Display for MessageLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageLevel::Error => write!(f, "Error"),
            MessageLevel::Warning => write!(f, "Warning"),
        }
    }
}
// $$
```

Location formatting for error messages, with 1-based line numbers for user display:

```rust
// <[chunk-location-impl]>=
impl ChunkLocation {
    fn format_message(&self, level: MessageLevel, msg: &str) -> String {
        // Add 1 to convert from 0-based to 1-based line numbers in displayed messages
        format!("{}: {} {}: {}", level, self.file, self.line + 1, msg)
    }
}
// $$
```

## Error Types

Various errors that can occur during chunk processing:

```rust
// <[chunk-error]>=
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
// $$
```

Error conversion implementations:

```rust
// <[chunk-error-impls]>=
impl std::fmt::Display for ChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkError::RecursionLimit { chunk, location } => write!(
                f,
                "{}",
                location.format_message(
                    MessageLevel::Error,
                    &format!(
                        "maximum recursion depth exceeded while expanding chunk '{}'",
                        chunk
                    )
                )
            ),
            ChunkError::RecursiveReference { chunk, location } => write!(
                f,
                "{}",
                location.format_message(
                    MessageLevel::Error,
                    &format!("recursive reference detected in chunk '{}'", chunk)
                )
            ),
            ChunkError::UndefinedChunk { chunk, location } => write!(
                f,
                "{}",
                location.format_message(
                    MessageLevel::Error,
                    &format!("referenced chunk '{}' is undefined", chunk)
                )
            ),
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

## Core Chunk Type

The Chunk struct that holds the content and metadata for each chunk:

```rust
// <[chunk-struct]>=
#[derive(Debug, Clone)]
struct Chunk {
    content: Vec<String>,
    base_indent: usize,
    location: ChunkLocation,
    reference_count: usize,
}

impl Chunk {
    fn new(base_indent: usize, file: String, line: usize) -> Self {
        Self {
            content: Vec::new(),
            base_indent,
            location: ChunkLocation { file, line },
            reference_count: 0,
        }
    }

    fn add_line(&mut self, line: String) {
        self.content.push(line);
    }

    fn increment_references(&mut self) {
        self.reference_count += 1;
    }
}
// $$
```

## Type Assembly

Finally, we assemble all the types into the main types-and-errors chunk:

```rust
// <[types-and-errors]>=
// <[chunk-location]>
// <[message-level]>
// <[chunk-location-impl]>
// <[chunk-error]>
// <[chunk-error-impls]>
// <[chunk-struct]>
// $$
```
