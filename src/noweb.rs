use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::writer::SafeFileWriter;

lazy_static! {
    static ref OPEN_RE: Regex = Regex::new(r"<<([^>]+)>>=").unwrap();
    static ref SLOT_RE: Regex = Regex::new(r"(\s*)<<([^>]+)>>\s*$").unwrap();
    static ref CLOSE_RE: Regex = Regex::new(r"@").unwrap();
}

#[derive(Debug)]
pub enum ChunkError {
    RecursionLimit(String),     // Hit maximum recursion depth
    RecursiveReference(String), // Found a recursive reference
    UndefinedChunk(String),     // Referenced chunk doesn't exist
    IoError(io::Error),         // Wrap std::io::Error
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

pub struct ChunkStore {
    chunks: HashMap<String, Vec<String>>,
    file_chunks: Vec<String>,
}

pub struct ChunkWriter<'a> {
    safe_file_writer: &'a mut SafeFileWriter,
}

pub struct Clip {
    store: ChunkStore,
    writer: SafeFileWriter,
}

impl ChunkStore {
    pub fn new() -> Self {
        ChunkStore {
            chunks: HashMap::new(),
            file_chunks: Vec::new(),
        }
    }

    pub fn read(&mut self, text: &str) {
        let mut chunk_name: Option<String> = None;

        for line in text.lines() {
            if let Some(captures) = OPEN_RE.captures(line) {
                chunk_name = Some(captures[1].to_string());
                self.chunks
                    .entry(chunk_name.clone().unwrap())
                    .or_insert_with(Vec::new);
                continue;
            }

            if CLOSE_RE.is_match(line) {
                chunk_name = None;
                continue;
            }

            if let Some(ref name) = chunk_name {
                if let Some(chunk_lines) = self.chunks.get_mut(name) {
                    chunk_lines.push(format!("{}\n", line));
                }
            }
        }

        self.file_chunks = self
            .chunks
            .keys()
            .filter(|name| name.starts_with("@file"))
            .cloned()
            .collect();
    }

    fn expand_with_depth(
        &self,
        chunk_name: &str,
        indent: &str,
        depth: usize,
        seen: &mut Vec<String>,
    ) -> Result<Vec<String>, ChunkError> {
        const MAX_DEPTH: usize = 100;
        if depth > MAX_DEPTH {
            return Err(ChunkError::RecursionLimit(chunk_name.to_string()));
        }

        if seen.contains(&chunk_name.to_string()) {
            return Err(ChunkError::RecursiveReference(chunk_name.to_string()));
        }
        seen.push(chunk_name.to_string());

        let mut result = Vec::new();

        if let Some(chunk_lines) = self.chunks.get(chunk_name) {
            for line in chunk_lines {
                if let Some(captures) = SLOT_RE.captures(line) {
                    let additional_indent = captures.get(1).map_or("", |m| m.as_str());
                    let referenced_chunk = captures.get(2).map_or("", |m| m.as_str());
                    let new_indent = format!("{}{}", indent, additional_indent);

                    result.extend(self.expand_with_depth(
                        referenced_chunk,
                        &new_indent,
                        depth + 1,
                        seen,
                    )?);
                } else {
                    if indent.is_empty() {
                        result.push(line.clone());
                    } else {
                        result.push(format!("{}{}", indent, line));
                    }
                }
            }
        } else {
            return Err(ChunkError::UndefinedChunk(chunk_name.to_string()));
        }

        seen.pop();
        Ok(result)
    }

    pub fn expand(&self, chunk_name: &str, indent: &str) -> Result<Vec<String>, ChunkError> {
        let mut seen = Vec::new();
        self.expand_with_depth(chunk_name, indent, 0, &mut seen)
    }

    pub fn get_file_chunks(&self) -> &[String] {
        &self.file_chunks
    }

    pub fn get_chunk_content(&self, chunk_name: &str) -> Option<&Vec<String>> {
        self.chunks.get(chunk_name)
    }

    #[cfg(test)]
    pub fn has_chunk(&self, name: &str) -> bool {
        self.chunks.contains_key(name)
    }
}

impl<'a> ChunkWriter<'a> {
    pub fn new(safe_file_writer: &'a mut SafeFileWriter) -> Self {
        ChunkWriter { safe_file_writer }
    }

    pub fn write_chunk(&mut self, chunk_name: &str, content: &[String]) -> Result<(), ChunkError> {
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
    pub fn new(safe_file_writer: SafeFileWriter) -> Self {
        Clip {
            store: ChunkStore::new(),
            writer: safe_file_writer,
        }
    }

    pub fn read(&mut self, text: &str) {
        self.store.read(text)
    }

    pub fn read_file<P: AsRef<Path>>(&mut self, input_path: P) -> Result<(), ChunkError> {
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

    pub fn write_files(&mut self) -> Result<(), ChunkError> {
        let mut writer = ChunkWriter::new(&mut self.writer);

        for file_chunk in self.store.get_file_chunks() {
            if let Some(content) = self.store.get_chunk_content(file_chunk) {
                writer.write_chunk(file_chunk, content)?;
            }
        }
        Ok(())
    }

    pub fn get_chunk<W: io::Write>(
        &self,
        chunk_name: &str,
        out_stream: &mut W,
    ) -> Result<(), ChunkError> {
        let content = self.store.expand(chunk_name, "")?;
        for line in content {
            out_stream.write_all(line.as_bytes())?;
        }
        out_stream.write_all(b"\n")?;
        Ok(())
    }

    pub fn expand(&self, chunk_name: &str, indent: &str) -> Result<Vec<String>, ChunkError> {
        self.store.expand(chunk_name, indent)
    }

    #[cfg(test)]
    pub fn has_chunk(&self, name: &str) -> bool {
        self.store.has_chunk(name)
    }

    #[cfg(test)]
    pub fn get_chunk_content(&self, name: &str) -> Option<&Vec<String>> {
        self.store.get_chunk_content(name)
    }

    #[cfg(test)]
    pub fn get_file_chunks(&self) -> &[String] {
        self.store.get_file_chunks()
    }
}
