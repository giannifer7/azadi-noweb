# ChunkWriter Implementation

## Writer Structure
The ChunkWriter manages safe file operations by delegating to SafeFileWriter:

```rust
// <[chunk-writer-struct]>=
pub struct ChunkWriter<'a> {
    safe_file_writer: &'a mut SafeFileWriter,
}

impl<'a> ChunkWriter<'a> {
    pub fn new(safe_file_writer: &'a mut SafeFileWriter) -> Self {
        ChunkWriter { safe_file_writer }
    }
}
// $$
```

## Writing Implementation
The core writing logic, with special handling for @file chunks:

```rust
// <[chunk-writer-impl]>=
impl<'a> ChunkWriter<'a> {
    pub fn write_chunk(&mut self, chunk_name: &str, content: &[String]) -> Result<(), AzadiError> {
        // Only process @file chunks
        if !chunk_name.starts_with("@file") {
            return Ok(());
        }

        // Extract filename, skipping the "@file " prefix
        let filename = &chunk_name[5..].trim();

        // Prepare the destination file
        let dest_filename = self.safe_file_writer.before_write(filename)?;

        // Write content atomically
        let mut file = fs::File::create(&dest_filename)?;
        for line in content {
            file.write_all(line.as_bytes())?;
        }

        // Finalize the write operation
        self.safe_file_writer.after_write(filename)?;
        Ok(())
    }
}
// $$
```

## Assembly
Putting the ChunkWriter components together:

```rust
// <[chunk-writer]>=
// <[chunk-writer-struct]>
// <[chunk-writer-impl]>
// $$
```
