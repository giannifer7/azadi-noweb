# chunkstore.nw

This document describes the implementation of the ChunkStore system for literate programming, with support for the `@replace` directive to allow chunk redefinition.

## Overview

The ChunkStore system manages and processes chunks of code, supporting expansion, recursion detection, and file operations. Each chunk is a named piece of code or text that can reference other chunks for expansion.

Key features:
- Parse input according to configurable delimiters and comment markers
- Track chunk locations for error reporting
- Handle chunk expansion with recursion and indentation control
- Support file output with path safety checks
- Allow chunk redefinition with `@replace`

## File Structure

Here's the main structure of our implementation:

```rust
// <[@file src/noweb.rs]>=
// src/noweb.rs
// <[imports]>
// <[types-and-errors]>
// <[chunk-store]>
// <[chunk-writer]>
// <[clip]>
// $$
```

## Base Dependencies

The imports we need for our implementation:

```rust
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
```
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

```rust
// <[chunk-store-new]>=
impl ChunkStore {
    pub fn new(
        open_delim: &str,
        close_delim: &str,
        chunk_end: &str,
        comment_markers: &[String],
    ) -> Self {
        let open_escaped = regex::escape(open_delim);
        let close_escaped = regex::escape(close_delim);

        // Escape each comment marker individually before joining
        let escaped_markers = comment_markers
            .iter()
            .map(|m| regex::escape(m))
            .collect::<Vec<_>>()
            .join("|");

        // Pattern for chunk opening, including @replace directive
        let open_pattern = format!(
            r"^(\s*)(?:{})?[ \t]*(?:@replace[ \t]+)?{}([^[:space:]{}]+){}=",
            escaped_markers,
            open_escaped,
            regex::escape(close_delim),  // Prevent delimiters in chunk names
            close_escaped
        );

        // Pattern for chunk reference in content
        let slot_pattern = format!(
            r"(\s*)(?:{})?[ \t]*{}([^[:space:]{}]+){}\s*$",
            escaped_markers,
            open_escaped,
            regex::escape(close_delim),
            close_escaped
        );

        // Pattern for chunk end marker
        let close_pattern = format!(
            r"^(?:{})?[ \t]*{}\s*$",
            escaped_markers,
            regex::escape(chunk_end)
        );

        ChunkStore {
            chunks: HashMap::new(),
            file_chunks: Vec::new(),
            open_re: Regex::new(&open_pattern).expect("Invalid open pattern"),
            slot_re: Regex::new(&slot_pattern).expect("Invalid slot pattern"),
            close_re: Regex::new(&close_pattern).expect("Invalid close pattern"),
        }
    }
}
// $$
```

## Chunk Name Validation

Validation for chunk names and paths:

```rust
// <[chunk-name-validation]>=
// Helper function to validate paths
fn path_is_safe(path: &str) -> Result<(), SafeWriterError> {
    let path = Path::new(path);

    // Check for absolute paths
    if path.is_absolute() {
        return Err(SafeWriterError::SecurityViolation(
            "Absolute paths are not allowed".to_string()
        ));
    }

    // Check for Windows-style paths
    if path.to_string_lossy().contains(':') {
        return Err(SafeWriterError::SecurityViolation(
            "Windows-style paths are not allowed".to_string()
        ));
    }

    // Check for path traversal
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(SafeWriterError::SecurityViolation(
            "Path traversal is not allowed".to_string()
        ));
    }

    Ok(())
}

impl ChunkStore {
    fn validate_chunk_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // If it's a file chunk, validate the path
        if name.starts_with("@file") {
            let path = name.trim_start_matches("@file").trim();
            return match path_is_safe(path) {
                Ok(_) => true,
                Err(_) => false,
            };
        }

        // Regular chunk name validation
        !name.contains(char::is_whitespace) &&
        !name.contains("<<") &&
        !name.contains(">>")
    }
}
// $$
```

## Reading Implementation

The core reading logic, now with support for @replace:

```rust
// <[chunk-store-read]>=
impl ChunkStore {
    pub fn read(&mut self, text: &str, file: &str) {
        let mut chunk_name: Option<String> = None;
        let mut line_number: i32 = -1;

        for line in text.lines() {
            line_number += 1;

            if let Some(captures) = self.open_re.captures(line) {
                let is_replace = line.contains("@replace");
                let indentation = captures.get(1).map_or("", |m| m.as_str());
                let name = captures.get(2).map_or("", |m| m.as_str()).to_string();

                // Only store valid chunk names
                if self.validate_chunk_name(&name) {
                    // If @replace is present, remove any existing chunk
                    if is_replace {
                        self.chunks.remove(&name);
                    }

                    chunk_name = Some(name.clone());
                    self.chunks.insert(
                        name,
                        Chunk::new(indentation.len(), file.to_string(), line_number as usize),
                    );
                }
                continue;
            }

            if self.close_re.is_match(line) {
                chunk_name = None;
                continue;
            }

            if let Some(ref name) = chunk_name {
                if let Some(chunk) = self.chunks.get_mut(name) {
                    if line.ends_with('\n') {
                        chunk.add_line(line.to_owned());
                    } else {
                        chunk.add_line(format!("{}\n", line));
                    }
                }
            }
        }

        self.file_chunks = self
            .chunks
            .keys()
            .filter(|name| name.starts_with("@file"))
            .map(String::to_owned)
            .collect();
    }
}
// $$
```

We'll continue with expansion and reference handling in the next chunk. Would you like me to proceed with that?
# Chunk Expansion and Reference Handling

## Depth-Aware Expansion

The core expansion logic that handles recursion detection and indentation:

```rust
// <[chunk-store-expand]>=
impl ChunkStore {
    pub fn expand_with_depth(
        &mut self,
        chunk_name: &str,
        target_indent: &str,
        depth: usize,
        seen: &mut Vec<(String, ChunkLocation)>,
        reference_location: ChunkLocation,
    ) -> Result<Vec<String>, ChunkError> {
        const MAX_DEPTH: usize = 100;
        if depth > MAX_DEPTH {
            return Err(ChunkError::RecursionLimit {
                chunk: chunk_name.to_owned(),
                location: reference_location,
            });
        }

        let chunk_owned = chunk_name.to_owned();
        if seen.iter().any(|(name, _)| name == &chunk_owned) {
            return Err(ChunkError::RecursiveReference {
                chunk: chunk_owned,
                location: reference_location,
            });
        }

        // First get required data from chunk without holding borrow
        let (content, base_indent, location) = {
            let chunk = match self.chunks.get(chunk_name) {
                Some(chunk) => chunk,
                None => {
                    return Err(ChunkError::UndefinedChunk {
                        chunk: chunk_name.to_owned(),
                        location: reference_location,
                    })
                }
            };
            (
                chunk.content.clone(),
                chunk.base_indent,
                chunk.location.clone(),
            )
        };

        // Now update reference count with a separate borrow
        if let Some(chunk) = self.chunks.get_mut(chunk_name) {
            chunk.increment_references();
        }

        seen.push((chunk_owned, reference_location));

        let mut result = Vec::new();
        let mut current_line = 0;

        for line in &content {
            current_line += 1;
            match self.slot_re.captures(line) {
                Some(captures) => {
                    let additional_indent = captures.get(1).map_or("", |m| m.as_str());
                    let referenced_chunk = captures.get(2).map_or("", |m| m.as_str());

                    let relative_indent = if additional_indent.len() > base_indent {
                        &additional_indent[base_indent..]
                    } else {
                        ""
                    };

                    let new_indent = if target_indent.is_empty() {
                        relative_indent.to_owned()
                    } else {
                        format!("{}{}", target_indent, relative_indent)
                    };

                    let expansion_location = ChunkLocation {
                        file: location.file.clone(),
                        line: location.line + current_line - 1,
                    };

                    let expanded = self.expand_with_depth(
                        referenced_chunk.trim(),
                        &new_indent,
                        depth + 1,
                        seen,
                        expansion_location,
                    )?;
                    result.extend(expanded);
                }
                None => {
                    let line_indent = if line.len() > base_indent {
                        &line[base_indent..]
                    } else {
                        line
                    };

                    if target_indent.is_empty() {
                        result.push(line_indent.to_owned());
                    } else {
                        result.push(format!("{}{}", target_indent, line_indent));
                    }
                }
            };
        }

        seen.pop();
        Ok(result)
    }
}
// $$
```

## Public Expansion Interface

The public interface for chunk expansion and content retrieval:

```rust
// <[chunk-store-public-expand]>=
impl ChunkStore {
    pub fn expand(&mut self, chunk_name: &str, indent: &str) -> Result<Vec<String>, ChunkError> {
        let mut seen = Vec::new();
        let initial_location = ChunkLocation {
            file: String::from("<root>"),
            line: 0,
        };
        self.expand_with_depth(chunk_name, indent, 0, &mut seen, initial_location)
    }

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
}
// $$
```

## Unused Chunk Detection

Warning generation for unused chunks:

```rust
// <[chunk-store-warnings]>=
impl ChunkStore {
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
}
// $$
```

## ChunkStore Assembly

Putting all the ChunkStore pieces together:

```rust
// <[chunk-store]>=
// <[chunk-store-struct]>
// <[chunk-name-validation]>
// <[chunk-store-new]>
// <[chunk-store-read]>
// <[chunk-store-expand]>
// <[chunk-store-public-expand]>
// <[chunk-store-warnings]>
// $$
```
# ChunkWriter Implementation

## Writer Structure
The ChunkWriter manages safe file operations by delegating to SafeFileWriter:

```rust
// <[chunk-writer-struct]>=
pub struct ChunkWriter<'a> {
    safe_file_writer: &'a mut SafeFileWriter,
}

impl<'a> ChunkWriter<'a> {
    pub fn new(safe_file_writer: &'a mut SafeFileWriter) -> Self {
        ChunkWriter { safe_file_writer }
    }
}
// $$
```

## Writing Implementation
The core writing logic, with special handling for @file chunks:

```rust
// <[chunk-writer-impl]>=
impl<'a> ChunkWriter<'a> {
    pub fn write_chunk(&mut self, chunk_name: &str, content: &[String]) -> Result<(), AzadiError> {
        // Only process @file chunks
        if !chunk_name.starts_with("@file") {
            return Ok(());
        }

        // Extract filename, skipping the "@file " prefix
        let filename = &chunk_name[5..].trim();

        // Prepare the destination file
        let dest_filename = self.safe_file_writer.before_write(filename)?;

        // Write content atomically
        let mut file = fs::File::create(&dest_filename)?;
        for line in content {
            file.write_all(line.as_bytes())?;
        }

        // Finalize the write operation
        self.safe_file_writer.after_write(filename)?;
        Ok(())
    }
}
// $$
```

## Assembly
Putting the ChunkWriter components together:

```rust
// <[chunk-writer]>=
// <[chunk-writer-struct]>
// <[chunk-writer-impl]>
// $$
```
# Clip Implementation

The Clip type provides the high-level interface to our literate programming system, combining ChunkStore and SafeFileWriter functionality.

## Main Structure

```rust
// <[clip-struct]>=
pub struct Clip {
    store: ChunkStore,
    writer: SafeFileWriter,
}
// $$
```

## Constructor and Basic Operations

```rust
// <[clip-new]>=
impl Clip {
    pub fn new(
        safe_file_writer: SafeFileWriter,
        open_delim: &str,
        close_delim: &str,
        chunk_end: &str,
        comment_markers: &[String],
    ) -> Self {
        Clip {
            store: ChunkStore::new(open_delim, close_delim, chunk_end, comment_markers),
            writer: safe_file_writer,
        }
    }

    pub fn reset(&mut self) {
        self.store.reset();
    }

    pub fn has_chunk(&self, name: &str) -> bool {
        self.store.has_chunk(name)
    }

    pub fn get_file_chunks(&self) -> Vec<String> {
        self.store.get_file_chunks().to_vec()
    }

    pub fn check_unused_chunks(&self) -> Vec<String> {
        self.store.check_unused_chunks()
    }
}
// $$
```

## Reading Implementation

Reading from strings and files:

```rust
// <[clip-read]>=
impl Clip {
    pub fn read(&mut self, text: &str, file: &str) {
        self.store.read(text, file)
    }

    pub fn read_file<P: AsRef<Path>>(&mut self, input_path: P) -> Result<(), AzadiError> {
        let content = fs::read_to_string(&input_path)?;
        self.store
            .read(&content, input_path.as_ref().to_string_lossy().as_ref());
        Ok(())
    }

    pub fn read_files<P: AsRef<Path>>(&mut self, input_paths: &[P]) -> Result<(), ChunkError> {
        for path in input_paths {
            self.read_file(path)?;
        }
        Ok(())
    }
}
// $$
```

## Writing and Expansion

File generation and content expansion:

```rust
// <[clip-write]>=
impl Clip {
    pub fn write_files(&mut self) -> Result<(), AzadiError> {
        // Store file chunk references to process
        let file_chunks: Vec<_> = self.store.get_file_chunks().to_vec();

        // Process all file chunks
        for file_chunk in &file_chunks {
            let expanded_content = self.expand(file_chunk, "")?;
            {
                let mut writer = ChunkWriter::new(&mut self.writer);
                writer.write_chunk(file_chunk, &expanded_content)?;
            }
        }

        // After processing, check for unused chunks and print warnings
        let warnings = self.store.check_unused_chunks();
        for warning in warnings {
            eprintln!("{}", warning);
        }

        Ok(())
    }

    pub fn get_chunk<W: io::Write>(
        &mut self,
        chunk_name: &str,
        out_stream: &mut W,
    ) -> Result<(), AzadiError> {
        let content = self.store.expand(chunk_name, "")?;
        for line in content {
            out_stream.write_all(line.as_bytes())?;
        }
        out_stream.write_all(b"\n")?;
        Ok(())
    }

    pub fn expand(&mut self, chunk_name: &str, indent: &str) -> Result<Vec<String>, AzadiError> {
        Ok(self.store.expand(chunk_name, indent)?)
    }

    pub fn get_chunk_content(&mut self, name: &str) -> Result<Vec<String>, ChunkError> {
        self.store.get_chunk_content(name)
    }
}
// $$
```

## Assembly
Putting all the Clip components together:

```rust
// <[clip]>=
// <[clip-struct]>
// <[clip-new]>
// <[clip-read]>
// <[clip-write]>
// $$
```

And with this, we have a complete implementation of our literate programming system, now with support for the `@replace` directive for chunk redefinition!
