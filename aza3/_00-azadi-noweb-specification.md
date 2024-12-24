# Azadi Noweb Specification

## Overview

Azadi Noweb is a literate programming tool that helps organize code into labeled sections called "chunks". 
The source code is embedded in Markdown files, and the tool processes these files to generate the actual
source code files.

## Etymology and Philosophy

The name "Azadi" comes from the Kurdish word "azadî" meaning "freedom". This reflects the tool's 
philosophy of giving developers the freedom to organize their code in ways that make sense for their 
specific needs, without imposing artificial constraints on file organization or content structure.

## Chunk Types and Syntax

### File Chunks
File chunks define the content of output files:
```rust
// <[@file path/to/file.rs]>=
// path/to/file.rs
actual code here
// $$
```

### Regular Chunks
Regular chunks contain reusable pieces of code or content:
```rust
// <[chunk-name]>=
content here
// $$
```

### Replacement Chunks
Replacement chunks replace all previous content of a chunk:
```rust
// <[@replace chunk-name]>=
new content here
// $$
```

### File Replacement Chunks
File replacement chunks replace entire file contents:
```rust
// <[@replace @file path/to/file.rs]>=
// path/to/file.rs
new file content
// $$
```

## Rules and Constraints

1. File Size and Format:
   - Each file must be smaller than 300 lines
   - Each line must be maximum 100 characters long
   - Files must be complete (no truncation or "..." ellipsis)

2. Chunk Names:
   - Must be unique across ALL files in the project
   - No spaces allowed in chunk names
   - Use hyphens or underscores instead of spaces
   - Names should be descriptive and consistent

3. File Paths:
   - Must be relative paths (no absolute paths)
   - Must not contain .. (to prevent path traversal)
   - Must use forward slashes (/) even on Windows
   - Must follow valid Rust module naming conventions when used for Rust files

4. Markdown Format:
   - All code must be within Markdown code blocks (```rust)
   - Documentation can go between code blocks
   - Code blocks must contain complete chunks
   - Each chunk must end with $$ on a separate line

5. Chunk Definition:
   - Multiple definitions of the same chunk append unless marked with @replace
   - Each chunk must have at least one definition
   - Chunks can reference other chunks
   - Each chunk definition and reference must be prefixed with // in Rust code blocks

## Development Workflow

1. File Organization:
   - Files are named with number prefixes for logical ordering (e.g., 00-, 01-, etc.)
   - Files must have .md extension
   - Files are processed in alphabetical order
   - Content can be organized freely across files based on what makes sense
   - Files should be sized and structured for clarity and maintainability

2. Chunk Names:
   - Must be unique across ALL files in the project
   - Cannot contain spaces (use hyphens or underscores)
   - Should be descriptive and meaningful

3. Updates:
   - Provided as new numbered files
   - Use replacement chunks to modify existing code
   - Must be self-contained
   - No manual file editing required

4. Testing Process:
   - Files are submitted to azadi-noweb
   - Output processed by Rust compiler
   - Results of cargo build/test shared back
   - Updates provided as new files if needed

## Example Usage and Indentation

Example of a complete file showing comment markers and indentation handling:

```rust
// <[@file src/lib.rs]>=
// src/lib.rs
// <[imports]>
pub struct Parser {
    // <[parser-fields]>
    
    pub fn new() -> Self {
        // <[parser-new]>
    }
}
// $$

// <[imports]>=
use std::fs;
use std::path::Path;
use std::io::{self, Read, Write};
// $$

// <[parser-fields]>=
chunks: HashMap<String, Vec<String>>,
current_file: Option<PathBuf>
// $$

// <[parser-new]>=
Parser {
    chunks: HashMap::new(),
    current_file: None,
}
// $$
```

Note how:
1. Each chunk definition starts with a comment marker (//) to preserve syntax highlighting
2. References to chunks also start with // when inside Rust code blocks
3. The `parser-fields` and `parser-new` chunks are referenced with indentation but defined at root level
4. The indentation from the reference site is preserved when the chunk content is expanded