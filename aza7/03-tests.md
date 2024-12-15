# Tests for Comment-Aware Chunk Processing

First, let's define our test constants with different comment styles:

````rust
<[test_constants]>=
// Test data with Python-style comments
const PYTHON_STYLE: &str = r#"
# <<test>>=
Hello
# @

# <<chunk1>>=
First chunk
# @

# <<chunk2>>=
Second chunk
# @
"#;

// Test data with Rust-style comments
const RUST_STYLE: &str = r#"
// <<test>>=
Hello
// @

// <<chunk1>>=
First chunk
// @

// <<chunk2>>=
Second chunk
// @
"#;

// Mixed styles in one file
const MIXED_STYLES: &str = r#"
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
$$

<[test_setup]>=
struct TestSetup {
    _temp_dir: TempDir,
    clip: Clip,
}

impl TestSetup {
    fn new(comment_markers: &[&str]) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let gen_path = temp_dir.path().join("gen");
        let private_path = temp_dir.path().join("private");
        fs::create_dir_all(&gen_path).unwrap();
        fs::create_dir_all(&private_path).unwrap();
        let safe_writer = SafeFileWriter::new(gen_path, private_path);

        let comment_markers = comment_markers.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let clip = Clip::new(safe_writer, "<<", ">>", "@", &comment_markers);

        TestSetup {
            _temp_dir: temp_dir,
            clip,
        }
    }
}
$$

Now let's write our tests:

````rust
<[comment_aware_tests]>=
#[test]
fn test_python_style_comments() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(PYTHON_STYLE);

    assert!(setup.clip.has_chunk("test"));
    assert!(setup.clip.has_chunk("chunk1"));
    assert!(setup.clip.has_chunk("chunk2"));

    assert_eq!(
        setup.clip.get_chunk_content("test").unwrap(),
        vec!["Hello\n"]
    );
}

#[test]
fn test_rust_style_comments() {
    let mut setup = TestSetup::new(&["//"]);
    setup.clip.read(RUST_STYLE);

    assert!(setup.clip.has_chunk("test"));
    assert_eq!(
        setup.clip.get_chunk_content("test").unwrap(),
        vec!["Hello\n"]
    );
}

#[test]
fn test_mixed_comment_styles() {
    let mut setup = TestSetup::new(&["#", "//"]);
    setup.clip.read(MIXED_STYLES);

    assert!(setup.clip.has_chunk("python_chunk"));
    assert!(setup.clip.has_chunk("rust_chunk"));

    let python_content = setup.clip.get_chunk_content("python_chunk").unwrap();
    assert!(python_content.join("").contains("print(\"Hello\")"));

    let rust_content = setup.clip.get_chunk_content("rust_chunk").unwrap();
    assert!(rust_content.join("").contains("println!(\"Hello\")"));
}

#[test]
fn test_indentation_with_comments() {
    let mut setup = TestSetup::new(&["#", "//"]);
    setup.clip.read(r#"
    # <<outer>>=
    before
        # <<inner>>
    after
    # @

    # <<inner>>=
    nested
    # @
    "#);

    let expanded = setup.clip.expand("outer", "").unwrap();
    assert_eq!(
        expanded,
        vec!["before\n", "    nested\n", "after\n"]
    );
}
$$

<[@file src/noweb_test.rs]>=
use std::fs;
use tempfile::TempDir;
use super::*;

<[test_constants]>

<[test_setup]>

<[comment_aware_tests]>

// Original tests remain...
$$
````

Now for the CLI updates:

<antArtifact identifier="04-cli" type="text/markdown" title="04-cli.md">
# CLI Updates for Comment Support

````rust
<[@file src/main.rs]>=
use clap::Parser;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use azadi_noweb::{AzadiError, Clip, SafeFileWriter};

#[derive(Parser)]
#[command(
    name = "azadi",
    about = "Expand chunks like noweb - A literate programming tool",
    version
)]
struct Args {
    /// Output file for --chunks [default: stdout]
    #[arg(long)]
    output: Option<PathBuf>,

    /// Names of chunks to extract (comma separated)
    #[arg(long)]
    chunks: Option<String>,

    /// Private work directory
    #[arg(long, default_value = "_azadi_work")]
    priv_dir: PathBuf,

    /// Base directory of generated files
    #[arg(long, default_value = "gen")]
    gen: PathBuf,

    /// Delimiter used to open a chunk
    #[arg(long, default_value = "<<")]
    open_delim: String,

    /// Delimiter used to close a chunk definition
    #[arg(long, default_value = ">>")]
    close_delim: String,

    /// Delimiter for chunk-end lines
    #[arg(long, default_value = "@")]
    chunk_end: String,

    /// Comment markers (comma separated)
    #[arg(long, default_value = "#,//")]
    comment_markers: String,

    /// Input files
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn run(args: Args) -> Result<(), AzadiError> {
    let comment_markers: Vec<String> = args.comment_markers
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let safe_writer = SafeFileWriter::new(&args.gen, &args.priv_dir);
    let mut clipper = Clip::new(
        safe_writer,
        &args.open_delim,
        &args.close_delim,
        &args.chunk_end,
        &comment_markers,
    );

    clipper.read_files(&args.files)?;
    clipper.write_files()?;

    if let Some(chunks) = args.chunks {
        let chunks: Vec<&str> = chunks.split(',').collect();
        if let Some(output_path) = args.output {
            let mut file = File::create(output_path)?;
            write_chunks(&clipper, &chunks, &mut file)?;
        } else {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            write_chunks(&clipper, &chunks, &mut handle)?;
        }
    }

    Ok(())
}

// ... rest of the file remains the same ...
$$
````

Finally, let's show some examples:

<antArtifact identifier="05-examples" type="text/markdown" title="05-examples.md">
# Examples of Using Comment-Aware Chunk Processing

## Python Example

````markdown
```python
# <[@file src/processor.py]>=
# src/processor.py
class DataProcessor:
    # <[init_method]>
    # <[process_method]>
# $$

# <[init_method]>=
def __init__(self):
    self.data = []
# $$

# <[process_method]>=
def process(self, items):
    for item in items:
        self.data.append(item.strip())
# $$
```
````

## Rust Example

````markdown
```rust
// <[@file src/lib.rs]>=
// src/lib.rs
pub struct Parser {
    // <[parser_fields]>
}

// <[parser_impl]>
// $$

// <[parser_fields]>=
source: String,
position: usize
// $$

// <[parser_impl]>=
impl Parser {
    pub fn new(source: String) -> Self {
        Self {
            source,
            position: 0
        }
    }
}
// $$
```
````

To process these files:

```bash
azadi-noweb --comment-markers="#,//" input.md
```
