# ChunkStore Implementation

```rust
// <[chunk-store-struct]>=
pub struct ChunkStore {
    chunks: HashMap<String, Chunk>,
    file_chunks: Vec<String>,
    open_re: Regex,
    slot_re: Regex,
    close_re: Regex,
}

// <[path-validation]>
// <[chunk-store-impl]>
// $$

// <[path-validation]>=
fn path_is_safe(path: &str) -> Result<(), SafeWriterError> {
    let path = Path::new(path);

    if path.is_absolute() {
        return Err(SafeWriterError::SecurityViolation(
            "Absolute paths are not allowed".to_string()
        ));
    }

    if path.to_string_lossy().contains(':') {
        return Err(SafeWriterError::SecurityViolation(
            "Windows-style paths are not allowed".to_string()
        ));
    }

    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(SafeWriterError::SecurityViolation(
            "Path traversal is not allowed".to_string()
        ));
    }

    Ok(())
}
// $$

// <[chunk-store-impl]>=
impl ChunkStore {
    // <[chunk-store-new]>
    // <[chunk-store-validate]>
    // <[chunk-store-read]>
    // <[chunk-store-expand]>
    // <[chunk-store-access]>
    // <[chunk-store-check-unused]>
}
// $$

// <[chunk-store-new]>=
pub fn new(
    open_delim: &str,
    close_delim: &str,
    chunk_end: &str,
    comment_markers: &[String],
) -> Self {
    let open_escaped = regex::escape(open_delim);
    let close_escaped = regex::escape(close_delim);

    let escaped_markers = comment_markers
        .iter()
        .map(|m| regex::escape(m))
        .collect::<Vec<_>>()
        .join("|");

    let open_pattern = format!(
        r"^(\s*)(?:{})?[ \t]*{}(?:@replace[ \t]+)?(?:@file[ \t]+)?([^\s]+){}=",
        escaped_markers,
        open_escaped,
        close_escaped
    );

    let slot_pattern = format!(
        r"(\s*)(?:{})?[ \t]*{}(?:@file[ \t]+)?([^\s]+){}\s*$",
        escaped_markers,
        open_escaped,
        close_escaped
    );

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
// $$

// <[chunk-store-validate]>=
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
// $$

// <[chunk-store-read]>=
pub fn read(&mut self, text: &str, file: &str) {
    let mut chunk_name: Option<String> = None;
    let mut line_number: i32 = -1;

    for line in text.lines() {
        line_number += 1;

        if let Some(captures) = self.open_re.captures(line) {
            let indentation = captures.get(1).map_or("", |m| m.as_str());
            let base_name = captures.get(2).map_or("", |m| m.as_str()).to_string();
            let is_replace = line.contains("@replace");
            let full_name = if line.contains("@file") {
                format!("@file {}", base_name)
            } else {
                base_name
            };
            
            if self.validate_chunk_name(&full_name) {
                if is_replace {
                    self.chunks.remove(&full_name);
                }
                
                chunk_name = Some(full_name.clone());
                self.chunks.insert(
                    full_name,
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
// $$
```