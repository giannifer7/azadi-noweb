use std::fs;
use tempfile::TempDir;

use super::*;

// Common test data to avoid repeated string allocations
const HELLO_CHUNK: &str = "
<<test>>=
Hello
@
";

const TWO_CHUNKS: &str = "
<<chunk1>>=
First chunk
@

<<chunk2>>=
Second chunk
@
";

const NESTED_CHUNKS: &str = "
<<outer>>=
Before
<<inner>>
After
@
<<inner>>=
Nested content
@
";

const INDENTED_CHUNK: &str = "
<<main>>=
    <<indented>>
@
<<indented>>=
some code
@
";

const FILE_CHUNKS: &str = "
<<@file output.txt>>=
content
@
<<other>>=
other content
@
";

const SIMPLE_LINES: &str = "
<<test>>=
Line 1
Line 2
@
";

const SEQUENTIAL_CHUNKS: &str = "
<<main>>=
<<part1>>
<<part2>>
@
<<part1>>=
First part
@
<<part2>>=
Second part
@
";

const PYTHON_CODE: &str = "
<<code>>=
def example():
    <<body>>
@
<<body>>=
print('hello')
@
";

struct TestSetup {
    _temp_dir: TempDir,
    clip: Clip,
}

impl TestSetup {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let gen_path = temp_dir.path().join("gen");
        let private_path = temp_dir.path().join("private");
        let safe_writer = SafeFileWriter::new(gen_path, private_path);
        let clip = Clip::new(safe_writer);

        TestSetup {
            _temp_dir: temp_dir,
            clip,
        }
    }
}

#[test]
fn test_basic_chunk_recognition() {
    let mut setup = TestSetup::new();
    setup.clip.read(HELLO_CHUNK);

    assert!(
        setup.clip.has_chunk("test"),
        "Should recognize chunk named 'test'"
    );
    assert_eq!(
        setup.clip.get_chunk_content("test").unwrap(),
        &vec!["Hello\n"]
    );
}

#[test]
fn test_multiple_chunks() {
    let mut setup = TestSetup::new();
    setup.clip.read(TWO_CHUNKS);

    assert!(setup.clip.has_chunk("chunk1"));
    assert!(setup.clip.has_chunk("chunk2"));
    assert_eq!(
        setup.clip.get_chunk_content("chunk1").unwrap(),
        &vec!["First chunk\n"]
    );
    assert_eq!(
        setup.clip.get_chunk_content("chunk2").unwrap(),
        &vec!["Second chunk\n"]
    );
}

#[test]
fn test_nested_chunk_expansion() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new();
    setup.clip.read(NESTED_CHUNKS);

    let expanded = setup.clip.expand("outer", "")?;
    let expected = vec!["Before\n", "Nested content\n", "After\n"];
    assert_eq!(expanded, expected);
    Ok(())
}

#[test]
fn test_indentation_preservation() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new();
    setup.clip.read(INDENTED_CHUNK);

    let expanded = setup.clip.expand("main", "")?;
    assert_eq!(expanded, vec!["    some code\n"]);
    Ok(())
}

#[test]
fn test_file_chunk_detection() {
    let mut setup = TestSetup::new();
    setup.clip.read(FILE_CHUNKS);

    let file_chunks = setup.clip.get_file_chunks();
    assert_eq!(file_chunks.len(), 1);
    let chunk_name = "@file output.txt";
    assert!(file_chunks.iter().any(|s| s == chunk_name));
}

#[test]
fn test_chunk_output_to_writer() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new();
    let mut output = Vec::new();

    setup.clip.read(SIMPLE_LINES);
    setup.clip.get_chunk("test", &mut output)?;

    let result = String::from_utf8(output).unwrap();
    assert_eq!(result, "Line 1\nLine 2\n\n");
    Ok(())
}

#[test]
fn test_multiple_chunk_references() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new();
    setup.clip.read(SEQUENTIAL_CHUNKS);

    let expanded = setup.clip.expand("main", "")?;
    let expected = vec!["First part\n", "Second part\n"];
    assert_eq!(expanded, expected);
    Ok(())
}

#[test]
fn test_file_writing() -> Result<(), ChunkError> {
    let temp = TempDir::new()?;
    let gen_path = temp.path().join("gen");
    let private_path = temp.path().join("private");
    let safe_writer = SafeFileWriter::new(gen_path.clone(), private_path);
    let mut clip = Clip::new(safe_writer);

    const FILE_CONTENT: &str = "
<<@file test.txt>>=
Hello, World!
@
";
    clip.read(FILE_CONTENT);
    clip.write_files()?;

    let content = fs::read_to_string(gen_path.join("test.txt"))?;
    assert_eq!(content.trim(), "Hello, World!");
    Ok(())
}

#[test]
fn test_multiple_file_generation() -> Result<(), ChunkError> {
    let temp = TempDir::new()?;
    let gen_path = temp.path().join("gen");
    let private_path = temp.path().join("private");
    let safe_writer = SafeFileWriter::new(gen_path.clone(), private_path);
    let mut clip = Clip::new(safe_writer);

    const TWO_FILES: &str = "
<<@file file1.txt>>=
Content 1
@
<<@file file2.txt>>=
Content 2
@
";
    clip.read(TWO_FILES);
    clip.write_files()?;

    let content1 = fs::read_to_string(gen_path.join("file1.txt"))?;
    let content2 = fs::read_to_string(gen_path.join("file2.txt"))?;

    assert_eq!(content1.trim(), "Content 1");
    assert_eq!(content2.trim(), "Content 2");
    Ok(())
}

#[test]
fn test_empty_chunk() {
    let mut setup = TestSetup::new();

    const EMPTY: &str = "
<<empty>>=
@
";
    setup.clip.read(EMPTY);

    assert!(setup.clip.has_chunk("empty"));
    assert!(setup.clip.get_chunk_content("empty").unwrap().is_empty());
}

#[test]
fn test_complex_indentation() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new();
    setup.clip.read(PYTHON_CODE);

    let expanded = setup.clip.expand("code", "")?;
    let expected = vec!["def example():\n", "    print('hello')\n"];
    assert_eq!(
        expanded, expected,
        "Indentation should be preserved from the chunk reference"
    );

    // Also test with additional base indentation
    let expanded_indented = setup.clip.expand("code", "  ")?;
    let expected_indented = vec!["  def example():\n", "      print('hello')\n"];
    assert_eq!(
        expanded_indented, expected_indented,
        "Both base and nested indentation should be preserved"
    );
    Ok(())
}

#[test]
fn test_recursive_chunk() {
    let mut setup = TestSetup::new();

    const RECURSIVE: &str = "
<<recursive>>=
Start
<<recursive>>
End
@
";
    setup.clip.read(RECURSIVE);

    let result = setup.clip.expand("recursive", "");
    assert!(matches!(
        result,
        Err(AzadiError::Chunk(ChunkError::RecursiveReference(_)))
    ));
}

#[test]
fn test_mutual_recursion() {
    let mut setup = TestSetup::new();

    const MUTUAL: &str = "
<<a>>=
A calls B:
<<b>>
@
<<b>>=
B calls A:
<<a>>
@
";
    setup.clip.read(MUTUAL);

    let result = setup.clip.expand("a", "");
    assert!(matches!(
        result,
        Err(AzadiError::Chunk(ChunkError::RecursiveReference(_)))
    ));
}

#[test]
fn test_reset() {
    let mut setup = TestSetup::new();

    setup.clip.read(
        "
<<test>>=
Hello
@
",
    );

    assert!(setup.clip.has_chunk("test"));

    setup.clip.reset();

    assert!(!setup.clip.has_chunk("test"));
    assert!(setup.clip.get_file_chunks().is_empty());
}
