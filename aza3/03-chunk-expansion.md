# Chunk Expansion Implementation

```rust
// <[chunk-store-expand]>=
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
        file: String::from("<root>"),
        line: 0,
    };
    self.expand_with_depth(chunk_name, indent, 0, &mut seen, initial_location)
}
// $$

// <[expand-depth-check]>=
const MAX_DEPTH: usize = 100;
if depth > MAX_DEPTH {
    return Err(ChunkError::RecursionLimit {
        chunk: chunk_name.to_owned(),
        location: reference_location,
    });
}
// $$

// <[expand-recursion-check]>=
let chunk_owned = chunk_name.to_owned();
if seen.iter().any(|(name, _)| name == &chunk_owned) {
    return Err(ChunkError::RecursiveReference {
        chunk: chunk_owned,
        location: reference_location,
    });
}
// $$

// <[expand-chunk-access]>=
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

if let Some(chunk) = self.chunks.get_mut(chunk_name) {
    chunk.increment_references();
}

seen.push((chunk_owned, reference_location));
// $$

// <[expand-content]>=
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
            } else