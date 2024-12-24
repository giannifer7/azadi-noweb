# Chunk Expansion Content

```rust
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
// $$
```