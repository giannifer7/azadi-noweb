# Noweb Implementation with Comment Support

First, let's define our error types for proper error handling:

````rust
<[error_types]>=
#[derive(Debug)]
pub enum ChunkError {
    RecursionLimit(String),
    RecursiveReference(String),
    UndefinedChunk(String),
    IoError(io::Error),
}

impl std::fmt::Display for ChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkError::RecursionLimit(chunk) => write!(
                f,
                "Maximum recursion depth exceeded while expanding chunk '{}'",
                chunk
            ),
            ChunkError::RecursiveReference(chunk) => {
                write!(f, "Recursive reference detected in chunk '{}'", chunk)
            }
            ChunkError::UndefinedChunk(chunk) => {
                write!(f, "Referenced chunk '{}' is undefined", chunk)
            }
            ChunkError::IoError(e) => write!(f, "I/O error: {}", e),
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
$$
````

Next, the core ChunkStore that handles chunk parsing and storage:

````rust
<[chunk_store]>=
pub struct ChunkStore {
    chunks: HashMap<String, Vec<String>>,
    file_chunks: Vec<String>,
    open_re: Regex,
    slot_re: Regex,
    close_re: Regex,
}
$$

The regex patterns are crucial for handling commented and uncommented chunk syntax:

````rust
<[regex_patterns]>=
// Pattern for chunk definitions: optional comment marker followed by <<name>>=
let open_pattern = format!(
    r"\s*(?:{})?[ \t]*{}(.+){}=",
    comment_markers.join("|"),
    open_escaped,
    close_escaped
);

// Pattern for chunk references: preserve indentation capture while allowing comments
let slot_pattern = format!(
    r"(\s*)(?:{})?[ \t]*{}(.+){}\s*$",
    comment_markers.join("|"),
    open_escaped,
    close_escaped
);

// Pattern for chunk end: optional comment marker followed by end marker
let close_pattern = format!(
    r"\s*(?:{})?[ \t]*{}\s*$",
    comment_markers.join("|"),
    regex::escape(chunk_end)
);
$$
````

The ChunkStore implementation:

````rust
<[chunk_store_implementation]>=
impl ChunkStore {
    pub fn new(
        open_delim: &str,
        close_delim: &str,
        chunk_end: &str,
        comment_markers: &[String]
    ) -> Self {
        let open_escaped = regex::escape(open_delim);
        let close_escaped = regex::escape(close_delim);

        <[regex_patterns]>

        ChunkStore {
            chunks: HashMap::new(),
            file_chunks: Vec::new(),
            open_re: Regex::new(&open_pattern).expect("Invalid open pattern"),
            slot_re: Regex::new(&slot_pattern).expect("Invalid slot pattern"),
            close_re: Regex::new(&close_pattern).expect("Invalid close pattern"),
        }
    }

    pub fn read(&mut self, text: &str) {
        let mut chunk_name: Option<String> = None;

        for line in text.lines() {
            if let Some(captures) = self.open_re.captures(line) {
                let name = captures[1].to_string();
                chunk_name = Some(name.clone());
                self.chunks.entry(name).or_default();
                continue;
            }

            if self.close_re.is_match(line) {
                chunk_name = None;
                continue;
            }

            if let Some(ref name) = chunk_name {
                if let Some(chunk_lines) = self.chunks.get_mut(name) {
                    if line.ends_with('\n') {
                        chunk_lines.push(line.to_owned());
                    } else {
                        chunk_lines.push(format!("{}\n", line));
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

    pub fn expand_with_depth(
        &self,
        chunk_name: &str,
        indent: &str,
        depth: usize,
        seen: &mut Vec<String>,
    ) -> Result<Vec<String>, ChunkError> {
        const MAX_DEPTH: usize = 100;
        if depth > MAX_DEPTH {
            return Err(ChunkError::RecursionLimit(chunk_name.to_owned()));
        }

        let chunk_owned = chunk_name.to_owned();
        if seen.contains(&chunk_owned) {
            return Err(ChunkError::RecursiveReference(chunk_owned));
        }
        seen.push(chunk_owned);

        let mut result = Vec::new();

        if let Some(chunk_lines) = self.chunks.get(chunk_name) {
            for line in chunk_lines {
                if let Some(captures) = self.slot_re.captures(line) {
                    let additional_indent = captures.get(1).map_or("", |m| m.as_str());
                    let referenced_chunk = captures.get(2).map_or("", |m| m.as_str());

                    let new_indent = if indent.is_empty() {
                        additional_indent.to_owned()
                    } else {
                        format!("{}{}", indent, additional_indent)
                    };

                    let expanded = self.expand_with_depth(
                        referenced_chunk.trim(),
                        &new_indent,
                        depth + 1,
                        seen,
                    )?;
                    result.extend(expanded);
                } else {
                    let mut new_line = line.clone();
                    if !indent.is_empty() {
                        let mut with_indent = String::with_capacity(indent.len() + new_line.len());
                        with_indent.push_str(indent);
                        with_indent.push_str(&new_line);
                        new_line = with_indent;
                    }
                    result.push(new_line);
                }
            }
        } else {
            return Err(ChunkError::UndefinedChunk(chunk_name.to_owned()));
        }

        seen.pop();
        Ok(result)
    }

    pub fn expand(&self, chunk_name: &str, indent: &str) -> Result<Vec<String>, ChunkError> {
        let mut seen = Vec::new();
        self.expand_with_depth(chunk_name, indent, 0, &mut seen)
    }

    pub fn get_chunk_content(&self, chunk_name: &str) -> Result<Vec<String>, ChunkError> {
        self.expand(chunk_name, "")
    }

    pub fn get_file_chunks(&self) -> &[String] {
        &self.file_chunks
    }

    pub fn reset(&mut self) {
        self.chunks.clear();
        self.file_chunks.clear();
    }

    #[cfg(test)]
    pub fn has_chunk(&self, name: &str) -> bool {
        self.chunks.contains_key(name)
    }
}
$$
````

Now let's put it all together in the main file:

````rust
<[@file src/noweb.rs]>=
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::AzadiError;
use crate::SafeFileWriter;

<[error_types]>

<[chunk_store]>

<[chunk_store_implementation]>

pub struct ChunkWriter<'a> {
    safe_file_writer: &'a mut SafeFileWriter,
}

pub struct Clip {
    store: ChunkStore,
    writer: SafeFileWriter,
}

impl<'a> ChunkWriter<'a> {
    pub fn new(safe_file_writer: &'a mut SafeFileWriter) -> Self {
        ChunkWriter { safe_file_writer }
    }

    pub fn write_chunk(&mut self, chunk_name: &str, content: &[String]) -> Result<(), AzadiError> {
        if !chunk_name.starts_with("@file") {
            return Ok(());
        }

        let filename = &chunk_name[5..].trim();
        let dest_filename = self.safe_file_writer.before_write(filename)?;

        let mut file = fs::File::create(&dest_filename)?;
        for line in content {
            file.write_all(line.as_bytes())?;
        }

        self.safe_file_writer.after_write(filename)?;
        Ok(())
    }
}

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

    pub fn read(&mut self, text: &str) {
        self.store.read(text)
    }

    pub fn read_file<P: AsRef<Path>>(&mut self, input_path: P) -> Result<(), AzadiError> {
        let content = fs::read_to_string(input_path)?;
        self.read(&content);
        Ok(())
    }

    pub fn read_files<P: AsRef<Path>>(&mut self, input_paths: &[P]) -> Result<(), ChunkError> {
        for path in input_paths {
            self.read_file(path)?;
        }
        Ok(())
    }

    pub fn write_files(&mut self) -> Result<(), AzadiError> {
        for file_chunk in self.store.get_file_chunks() {
            let expanded_content = self.expand(file_chunk, "")?;
            {
                let mut writer = ChunkWriter::new(&mut self.writer);
                writer.write_chunk(file_chunk, &expanded_content)?;
            }
        }

        Ok(())
    }

    pub fn get_chunk<W: io::Write>(
        &self,
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

    pub fn expand(&self, chunk_name: &str, indent: &str) -> Result<Vec<String>, AzadiError> {
        Ok(self.store.expand(chunk_name, indent)?)
    }

    pub fn reset(&mut self) {
        self.store.reset();
    }

    #[cfg(test)]
    pub fn has_chunk(&self, name: &str) -> bool {
        self.store.has_chunk(name)
    }

    #[cfg(test)]
    pub fn get_chunk_content(&self, name: &str) -> Result<Vec<String>, ChunkError> {
        self.store.get_chunk_content(name)
    }

    #[cfg(test)]
    pub fn get_file_chunks(&self) -> &[String] {
        self.store.get_file_chunks()
    }
}
$$
````

Would you like me to continue with:
1. The test file updates?
2. The CLI changes for comment markers?
3. An example of how to use the new comment-aware functionality?