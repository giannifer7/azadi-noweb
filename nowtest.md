# Common Test Infrastructure
This file defines the basic test infrastructure and fixtures shared across our test files.

```rust
// <[@file src/test_common.rs]>=
use super::*;
use std::fs;
use tempfile::TempDir;

// <[test-setup]>
// <[test-fixtures]>
// <[test-helpers]>
// $$
```

## Basic Test Setup

```rust
// <[test-setup]>=
pub struct TestSetup {
    pub _temp_dir: TempDir,
    pub clip: Clip,
}

impl TestSetup {
    pub fn new(comment_markers: &[&str]) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let gen_path = temp_dir.path().join("gen");
        let private_path = temp_dir.path().join("private");
        fs::create_dir_all(&gen_path).unwrap();
        fs::create_dir_all(&private_path).unwrap();
        let safe_writer = SafeFileWriter::new(gen_path, private_path);

        let comment_markers = comment_markers
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let clip = Clip::new(safe_writer, "<<", ">>", "@", &comment_markers);

        TestSetup {
            _temp_dir: temp_dir,
            clip,
        }
    }
}
// $$

## Test Fixtures
All the sample chunks used across different tests:

```rust
// <[test-fixtures]>=
pub const BASIC_CHUNK: &str = r#"
# <<test>>=
Hello
# @
"#;

pub const TWO_CHUNKS: &str = r#"
# <<chunk1>>=
First chunk
# @
# <<chunk2>>=
Second chunk
# @
"#;

pub const NESTED_CHUNKS: &str = r#"
# <<outer>>=
Before
# <<inner>>
After
# @
# <<inner>>=
Nested content
# @
"#;

pub const INDENTED_CHUNK: &str = r#"
# <<main>>=
    # <<indented>>
# @
# <<indented>>=
some code
# @
"#;

pub const PYTHON_CODE: &str = r#"
# <<code>>=
def example():
    # <<body>>
# @
# <<body>>=
print('hello')
# @
"#;
// $$
```

Would you like me to continue with the rest of the fixtures and start with the basic tests file? I'm keeping each section focused and under 300 lines.
# More Test Fixtures
Adding the remaining fixtures to test_common.rs:

```rust
// <[@replace test-fixtures]>=
pub const BASIC_CHUNK: &str = r#"
# <<test>>=
Hello
# @
"#;

pub const TWO_CHUNKS: &str = r#"
# <<chunk1>>=
First chunk
# @
# <<chunk2>>=
Second chunk
# @
"#;

pub const NESTED_CHUNKS: &str = r#"
# <<outer>>=
Before
# <<inner>>
After
# @
# <<inner>>=
Nested content
# @
"#;

pub const INDENTED_CHUNK: &str = r#"
# <<main>>=
    # <<indented>>
# @
# <<indented>>=
some code
# @
"#;

pub const PYTHON_CODE: &str = r#"
# <<code>>=
def example():
    # <<body>>
# @
# <<body>>=
print('hello')
# @
"#;

pub const MULTI_COMMENT_CHUNKS: &str = r#"
# <<python_chunk>>=
def hello():
    print("Hello")
# @

// <<rust_chunk>>=
fn main() {
    println!("Hello");
}
// @
"#;

pub const FILE_CHUNKS: &str = r#"
# <<@file output.txt>>=
content
# @
# <<other>>=
other content
# @
"#;

pub const TWO_FILES: &str = r#"
# <<@file file1.txt>>=
Content 1
# @
# <<@file file2.txt>>=
Content 2
# @
"#;

pub const SEQUENTIAL_CHUNKS: &str = r#"
# <<main>>=
# <<part1>>
# <<part2>>
# @
# <<part1>>=
First part
# @
# <<part2>>=
Second part
# @
"#;

pub const EMPTY_CHUNK: &str = r#"
# <<empty>>=
# @
"#;
// $$
```

Now let's start with the basic tests file:

```rust
// <[@file src/noweb_new_test.rs]>=
mod test_common;
use test_common::*;

// <[basic-chunk-tests]>
// <[indentation-tests]>
// <[comment-style-tests]>
// $$
```

Let's implement the basic chunk tests:

```rust
// <[basic-chunk-tests]>=
#[test]
fn test_basic_chunk() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(BASIC_CHUNK, "test_basic.nw");

    assert!(setup.clip.has_chunk("test"));
    assert_eq!(
        setup.clip.get_chunk_content("test").unwrap(),
        vec!["Hello\n"]
    );
}

#[test]
fn test_multiple_chunks() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(TWO_CHUNKS, "test_multiple.nw");

    assert!(setup.clip.has_chunk("chunk1"));
    assert!(setup.clip.has_chunk("chunk2"));
    assert_eq!(
        setup.clip.get_chunk_content("chunk1").unwrap(),
        vec!["First chunk\n"]
    );
    assert_eq!(
        setup.clip.get_chunk_content("chunk2").unwrap(),
        vec!["Second chunk\n"]
    );
}

#[test]
fn test_nested_chunk_expansion() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(NESTED_CHUNKS, "test_nested.nw");

    let expanded = setup.clip.expand("outer", "")?;
    let expected = vec!["Before\n", "Nested content\n", "After\n"];
    assert_eq!(expanded, expected, "Nested chunks should expand correctly");
    Ok(())
}
// $$
```
# Basic Test Suite
Continuing with indentation and comment style tests:

```rust
// <[indentation-tests]>=
#[test]
fn test_indentation_preservation() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(INDENTED_CHUNK, "test_indent.nw");

    let expanded = setup.clip.expand("main", "")?;
    assert_eq!(
        expanded,
        vec!["    some code\n"],
        "Indentation should be preserved"
    );
    Ok(())
}

#[test]
fn test_complex_indentation() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(PYTHON_CODE, "test_python.nw");

    let expanded = setup.clip.expand("code", "")?;
    let expected = vec!["def example():\n", "    print('hello')\n"];
    assert_eq!(expanded, expected);

    // Test with additional base indentation
    let expanded_indented = setup.clip.expand("code", "  ")?;
    let expected_indented = vec!["  def example():\n", "      print('hello')\n"];
    assert_eq!(expanded_indented, expected_indented);
    Ok(())
}
// $$

```rust
// <[comment-style-tests]>=
#[test]
fn test_multi_comment_styles() {
    let mut setup = TestSetup::new(&["#", "//"]);
    setup.clip.read(MULTI_COMMENT_CHUNKS, "test_comments.nw");

    assert!(setup.clip.has_chunk("python_chunk"));
    assert!(setup.clip.has_chunk("rust_chunk"));

    let python_content = setup.clip.get_chunk_content("python_chunk").unwrap();
    assert!(python_content.join("").contains("print(\"Hello\")"));

    let rust_content = setup.clip.get_chunk_content("rust_chunk").unwrap();
    assert!(rust_content.join("").contains("println!(\"Hello\")"));
}

#[test]
fn test_empty_chunk() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(EMPTY_CHUNK, "test_empty.nw");

    assert!(setup.clip.has_chunk("empty"));
    assert!(
        setup.clip.get_chunk_content("empty").unwrap().is_empty(),
        "empty chunk should have no content"
    );
}

#[test]
fn test_sequential_chunks() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(SEQUENTIAL_CHUNKS, "test_sequential.nw");

    let expanded = setup.clip.expand("main", "")?;
    assert_eq!(expanded, vec!["First part\n", "Second part\n"]);
    Ok(())
}
// $$
```
# Advanced Tests Organization

## File Structure for Advanced Tests

```rust
// <[@file src/noweb_advanced_test.rs]>=
mod test_common;
use test_common::*;

// File operation tests
// <[file-tests]>

// Error handling tests
// <[error-tests]>

// Recursion detection tests
// <[recursion-tests]>

// Security and safety tests
// <[safety-tests]>
// $$
```

## File Operation Tests Section

```rust
// <[file-tests]>=
// <[file-detection-tests]>
// <[file-writing-tests]>
// <[file-generation-tests]>
// $$
```

## Error Tests Section

```rust
// <[error-tests]>=
// <[undefined-chunk-tests]>
// <[recursive-chunk-tests]>
// <[max-recursion-tests]>
// <[error-message-tests]>
// $$
```

## Recursion Tests Section

```rust
// <[recursion-tests]>=
// <[mutual-recursion-tests]>
// <[complex-recursion-tests]>
// <[diamond-dependency-tests]>
// $$
```

## Safety Tests Section

```rust
// <[safety-tests]>=
// <[dangerous-markers-tests]>
// <[regex-safety-tests]>
// <[name-validation-tests]>
// <[unicode-safety-tests]>
// $$
```

Would you like me to continue with implementing each section's content? I'll start with the file detection tests.