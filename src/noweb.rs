use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use crate::writer::SafeFileWriter;

lazy_static! {
    static ref OPEN_RE: Regex = Regex::new(r"<<([^>]+)>>=").unwrap();
    static ref SLOT_RE: Regex = Regex::new(r"(\s*)<<([^>]+)>>\s*$").unwrap();
    static ref CLOSE_RE: Regex = Regex::new(r"@").unwrap();
}

pub struct Clip {
    safe_file_writer: SafeFileWriter,
    chunks: HashMap<String, Vec<String>>,
    file_chunks: Vec<String>,
}

impl Clip {
    pub fn new(safe_file_writer: SafeFileWriter) -> Self {
        Clip {
            safe_file_writer,
            chunks: HashMap::new(),
            file_chunks: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn has_chunk(&self, name: &str) -> bool {
        self.chunks.contains_key(name)
    }

    #[cfg(test)]
    pub fn get_chunk_content(&self, name: &str) -> Option<&Vec<String>> {
        self.chunks.get(name)
    }

    #[cfg(test)]
    pub fn get_file_chunks(&self) -> &[String] {
        &self.file_chunks
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

    pub fn expand(&self, chunk_name: &str, base_indent: &str) -> Vec<String> {
        let mut result = Vec::new();

        if let Some(chunk_lines) = self.chunks.get(chunk_name) {
            for line in chunk_lines {
                if let Some(captures) = SLOT_RE.captures(line) {
                    let line_indent = captures.get(1).map_or("", |m| m.as_str());
                    let referenced_chunk = captures.get(2).map_or("", |m| m.as_str());

                    // Combine the base indentation with the line's own indentation
                    let combined_indent = format!("{}{}", base_indent, line_indent);
                    result.extend(self.expand(referenced_chunk, &combined_indent));
                } else {
                    // For regular lines, just add the base indentation
                    let indented_line = if base_indent.is_empty() {
                        line.to_string()
                    } else {
                        format!("{}{}", base_indent, line)
                    };
                    result.push(indented_line);
                }
            }
        } else {
            eprintln!("Undefined chunk: {}", chunk_name);
        }

        result
    }

    pub fn get_chunk<W: io::Write>(
        &self,
        output_chunk_name: &str,
        out_stream: &mut W,
    ) -> io::Result<()> {
        for line in self.expand(output_chunk_name, "") {
            out_stream.write_all(line.as_bytes())?;
        }
        out_stream.write_all(b"\n")?;
        Ok(())
    }

    pub fn read_file<P: AsRef<Path>>(&mut self, input_path: P) -> io::Result<()> {
        let content = fs::read_to_string(input_path)?;
        self.read(&content);
        Ok(())
    }

    pub fn read_files<P: AsRef<Path>>(&mut self, input_paths: &[P]) -> io::Result<()> {
        for path in input_paths {
            self.read_file(path)?;
        }
        Ok(())
    }

    pub fn write_file(&mut self, file_chunk: &str) -> io::Result<()> {
        let filename = file_chunk[5..].trim();
        let dest_filename = self.safe_file_writer.before_write(filename)?;

        if dest_filename.as_os_str().is_empty() {
            return Ok(());
        }

        let mut file = fs::File::create(&dest_filename)?;
        self.get_chunk(file_chunk, &mut file)?;

        self.safe_file_writer.after_write(filename)?;
        Ok(())
    }

    pub fn write_files(&mut self) -> io::Result<()> {
        let chunks_to_process: Vec<String> = self.file_chunks.clone();
        for file_chunk in chunks_to_process {
            self.write_file(&file_chunk)?;
        }
        Ok(())
    }
}
