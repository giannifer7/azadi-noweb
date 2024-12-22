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
