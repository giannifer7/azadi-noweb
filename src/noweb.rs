use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::AzadiError;
use crate::SafeFileWriter;

#[derive(Debug, Clone)]
pub struct ChunkLocation {
    pub file: String,
    pub line: usize,
}

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

impl ChunkLocation {
    fn format_message(&self, level: MessageLevel, msg: &str) -> String {
        // Add 1 to convert from 0-based to 1-based line numbers in displayed messages
        format!("{}: {} {}: {}", level, self.file, self.line + 1, msg)
    }
}

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

pub struct ChunkStore {
    chunks: HashMap<String, Chunk>,
    file_chunks: Vec<String>,
    open_re: Regex,
    slot_re: Regex,
    close_re: Regex,
}

pub struct ChunkWriter<'a> {
    safe_file_writer: &'a mut SafeFileWriter,
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
impl ChunkStore {
    pub fn new(
        open_delim: &str,
        close_delim: &str,
        chunk_end: &str,
        comment_markers: &[String],
    ) -> Self {
        let open_escaped = regex::escape(open_delim);
        let close_escaped = regex::escape(close_delim);

        let open_pattern = format!(
            r"^(\s*)(?:{})?[ \t]*{}(.+){}=",
            comment_markers.join("|"),
            open_escaped,
            close_escaped
        );

        let slot_pattern = format!(
            r"(\s*)(?:{})?[ \t]*{}(.+){}\s*$",
            comment_markers.join("|"),
            open_escaped,
            close_escaped
        );

        let close_pattern = format!(
            r"^(?:{})?[ \t]*{}\s*$",
            comment_markers.join("|"),
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

    pub fn read(&mut self, text: &str, file: &str) {
        let mut chunk_name: Option<String> = None;
        let mut line_number: i32 = -1;

        for line in text.lines() {
            line_number += 1;

            if let Some(captures) = self.open_re.captures(line) {
                let indentation = captures.get(1).map_or("", |m| m.as_str());
                let name = captures.get(2).map_or("", |m| m.as_str()).to_string();
                chunk_name = Some(name.clone());
                self.chunks.insert(
                    name,
                    Chunk::new(indentation.len(), file.to_string(), line_number as usize),
                );
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

pub struct Clip {
    store: ChunkStore,
    writer: SafeFileWriter,
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

    pub fn reset(&mut self) {
        self.store.reset();
    }

    pub fn has_chunk(&self, name: &str) -> bool {
        self.store.has_chunk(name)
    }

    pub fn get_chunk_content(&mut self, name: &str) -> Result<Vec<String>, ChunkError> {
        self.store.get_chunk_content(name)
    }

    pub fn get_file_chunks(&self) -> Vec<String> {
        self.store.get_file_chunks().to_vec()
    }

    pub fn check_unused_chunks(&self) -> Vec<String> {
        self.store.check_unused_chunks()
    }
}
