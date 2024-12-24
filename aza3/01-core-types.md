# Core Types and Error Handling

```rust
// <[@file src/noweb.rs]>=
// src/noweb.rs
// <[imports]>
// <[file-store]>
// <[chunk-location]>
// <[message-level]>
// <[chunk-error]>
// <[chunk-struct]>
// <[chunk-store-struct]>
// <[chunk-writer]>
// <[clip-struct]>
// $$

// <[imports]>=
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, Component};

use crate::AzadiError;
use crate::SafeFileWriter;
use crate::SafeWriterError;
// $$

// <[chunk-location]>=
#[derive(Debug, Clone)]
pub struct ChunkLocation {
    pub file: String,
    pub line: usize,
}

impl ChunkLocation {
    fn format_message(&self, level: MessageLevel, msg: &str) -> String {
        // Add 1 to convert from 0-based to 1-based line numbers in displayed messages
        format!("{}: {} {}: {}", level, self.file, self.line + 1, msg)
    }
}
// $$

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

// <[chunk-error-impls]>
// $$

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