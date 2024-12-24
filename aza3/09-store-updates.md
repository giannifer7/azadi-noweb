# ChunkStore Updates

```rust
// <[@replace chunk-store-struct]>=
pub struct ChunkStore {
    chunks: HashMap<String, Chunk>,
    file_chunks: Vec<String>,
    files: FileStore,
    open_re: Regex,
    slot_re: Regex,
    close_re: Regex,
}
// $$

// <[@replace chunk-store-new]>=
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
        files: FileStore::new(),
        open_re: Regex::new(&open_pattern).expect("Invalid open pattern"),
        slot_re: Regex::new(&slot_pattern).expect("Invalid slot pattern"),
        close_re: Regex::new(&close_pattern).expect("Invalid close pattern"),
    }
}
// $$

// <[@replace chunk-store-read]>=
pub fn read(&mut self, text: &str, file: &str) {
    let file_index = self.files.add_file(file);
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
                    Chunk::new(indentation.len(), file_index, line_number as usize),
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

// <[@replace chunk-store-check-unused]>=
pub fn check_unused_chunks(&self) -> Vec<String> {
    let mut warnings = Vec::new();
    for (name, chunk) in &self.chunks {
        if !name.starts_with("@file") && chunk.reference_count == 0 {
            warnings.push(chunk.location.format_message(
                &self.files,
                MessageLevel::Warning,
                &format!("chunk '{}' is defined but never referenced", name),
            ));
        }
    }
    warnings.sort();
    warnings
}
// $$
```