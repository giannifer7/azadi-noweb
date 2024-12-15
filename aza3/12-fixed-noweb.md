# Core Library Implementation

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

<[chunk_writer]>

<[chunk_store_implementation]>

<[clip_implementation]>
$$

<[chunk_writer]>=
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
$$
````

[Content continues from previous files for other chunks...]

Would you like me to:
1. Show the updated error types and other chunks?
2. Update any related interfaces that might be affected?
3. Show how ChunkWriter integrates with the rest of the implementation?