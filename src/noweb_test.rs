// noweb_test.rs

use std::fs;
use tempfile::TempDir;

use super::*;

// Common test data to avoid repeated string allocations
const HELLO_CHUNK: &str = r#"
<<test>>=
Hello
@
"#;

const TWO_CHUNKS: &str = r#"
<<chunk1>>=
First chunk
@
<<chunk2>>=
Second chunk
@
"#;

const NESTED_CHUNKS: &str = r#"
<<outer>>=
Before
<<inner>>
After
@
<<inner>>=
Nested content
@
"#;

const INDENTED_CHUNK: &str = r#"
<<main>>=
    <<indented>>
@
<<indented>>=
some code
@
"#;

const FILE_CHUNKS: &str = r#"
<<@file output.txt>>=
content
@
<<other>>=
other content
@
"#;

const SIMPLE_LINES: &str = r#"
<<test>>=
Line 1
Line 2
@
"#;

const SEQUENTIAL_CHUNKS: &str = r#"
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
"#;

const PYTHON_CODE: &str = r#"
<<code>>=
def example():
    <<body>>
@
<<body>>=
print('hello')
@
"#;

// Structure to set up tests with temporary directories and Clip instances
struct TestSetup {
    _temp_dir: TempDir, // Keeps the temporary directory alive for the duration of the test
    clip: Clip,
}

impl TestSetup {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let gen_path = temp_dir.path().join("gen");
        let private_path = temp_dir.path().join("private");
        fs::create_dir_all(&gen_path).unwrap();
        fs::create_dir_all(&private_path).unwrap();
        let safe_writer = SafeFileWriter::new(gen_path, private_path);

        // Initialize Clip with default delimiters
        let clip = Clip::new(safe_writer, "<<", ">>", "@");

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
        vec!["Hello\n"]
    );
}

#[test]
fn test_multiple_chunks() {
    let mut setup = TestSetup::new();
    setup.clip.read(TWO_CHUNKS);

    assert!(setup.clip.has_chunk("chunk1"), "chunk1 should be present");
    assert!(setup.clip.has_chunk("chunk2"), "chunk2 should be present");
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
    let mut setup = TestSetup::new();
    setup.clip.read(NESTED_CHUNKS);

    let expanded = setup.clip.expand("outer", "")?;
    let expected = vec!["Before\n", "Nested content\n", "After\n"];
    assert_eq!(expanded, expected, "Nested chunks should expand correctly");
    Ok(())
}

#[test]
fn test_indentation_preservation() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new();
    setup.clip.read(INDENTED_CHUNK);

    let expanded = setup.clip.expand("main", "")?;
    assert_eq!(
        expanded,
        vec!["    some code\n"],
        "Indentation should be preserved"
    );
    Ok(())
}

#[test]
fn test_file_chunk_detection() {
    let mut setup = TestSetup::new();
    setup.clip.read(FILE_CHUNKS);

    let file_chunks = setup.clip.get_file_chunks();
    assert_eq!(file_chunks.len(), 1, "There should be one @file chunk");
    let chunk_name = "@file output.txt".to_string();
    assert!(
        file_chunks.contains(&chunk_name),
        "file_chunks should contain '@file output.txt'"
    );
}

#[test]
fn test_chunk_output_to_writer() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new();
    let mut output = Vec::new();

    setup.clip.read(SIMPLE_LINES);
    setup.clip.get_chunk("test", &mut output)?;

    let result = String::from_utf8(output).unwrap();
    assert_eq!(
        result, "Line 1\nLine 2\n\n",
        "Chunk output should match expected lines"
    );
    Ok(())
}

#[test]
fn test_multiple_chunk_references() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new();
    setup.clip.read(SEQUENTIAL_CHUNKS);

    let expanded = setup.clip.expand("main", "")?;
    let expected = vec!["First part\n", "Second part\n"];
    assert_eq!(
        expanded, expected,
        "Multiple chunk references should expand correctly"
    );
    Ok(())
}

#[test]
fn test_file_writing() -> Result<(), ChunkError> {
    let temp = TempDir::new()?;
    let gen_path = temp.path().join("gen");
    let private_path = temp.path().join("private");
    fs::create_dir_all(&gen_path)?;
    fs::create_dir_all(&private_path)?;
    let safe_writer = SafeFileWriter::new(gen_path.clone(), private_path);

    // Initialize Clip with default delimiters
    let mut clip = Clip::new(safe_writer, "<<", ">>", "@");

    const FILE_CONTENT: &str = r#"
<<@file test.txt>>=
Hello, World!
@
"#;
    clip.read(FILE_CONTENT);
    assert!(
        clip.has_chunk("@file test.txt"),
        "@file test.txt chunk should be present"
    );

    clip.write_files()?;

    let content = fs::read_to_string(gen_path.join("test.txt"))?;
    assert_eq!(
        content.trim(),
        "Hello, World!",
        "test.txt should have correct content"
    );
    Ok(())
}

#[test]
fn test_multiple_file_generation() -> Result<(), ChunkError> {
    let temp = TempDir::new()?;
    let gen_path = temp.path().join("gen");
    let private_path = temp.path().join("private");
    fs::create_dir_all(&gen_path)?;
    fs::create_dir_all(&private_path)?;
    let safe_writer = SafeFileWriter::new(gen_path.clone(), private_path);

    // Initialize Clip with default delimiters
    let mut clip = Clip::new(safe_writer, "<<", ">>", "@");

    const TWO_FILES: &str = r#"
<<@file file1.txt>>=
Content 1
@
<<@file file2.txt>>=
Content 2
@
"#;
    clip.read(TWO_FILES);
    clip.write_files()?;

    let content1 = fs::read_to_string(gen_path.join("file1.txt"))?;
    let content2 = fs::read_to_string(gen_path.join("file2.txt"))?;

    assert_eq!(
        content1.trim(),
        "Content 1",
        "file1.txt should have correct content"
    );
    assert_eq!(
        content2.trim(),
        "Content 2",
        "file2.txt should have correct content"
    );
    Ok(())
}
#[test]
fn test_empty_chunk() {
    let mut setup = TestSetup::new();

    const EMPTY: &str = r#"
<<empty>>=
@
"#;
    setup.clip.read(EMPTY);

    assert!(
        setup.clip.has_chunk("empty"),
        "empty chunk should be present"
    );
    assert!(
        setup.clip.get_chunk_content("empty").unwrap().is_empty(),
        "empty chunk should have no content"
    );
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

    const RECURSIVE: &str = r#"
<<recursive>>=
Start
<<recursive>>
End
@
"#;
    setup.clip.read(RECURSIVE);

    let result = setup.clip.expand("recursive", "");
    assert!(
        matches!(
            result,
            Err(AzadiError::Chunk(ChunkError::RecursiveReference(_)))
        ),
        "Recursive reference should be detected"
    );
}

#[test]
fn test_mutual_recursion() {
    let mut setup = TestSetup::new();

    const MUTUAL: &str = r#"
<<a>>=
A calls B:
<<b>>
@
<<b>>=
B calls A:
<<a>>
@
"#;
    setup.clip.read(MUTUAL);

    let result = setup.clip.expand("a", "");
    assert!(
        matches!(
            result,
            Err(AzadiError::Chunk(ChunkError::RecursiveReference(_)))
        ),
        "Mutual recursion should be detected"
    );
}

#[test]
fn test_reset() {
    let mut setup = TestSetup::new();

    setup.clip.read(
        r#"
<<test>>=
Hello
@
"#,
    );

    assert!(setup.clip.has_chunk("test"), "test chunk should be present");

    setup.clip.reset();

    assert!(
        !setup.clip.has_chunk("test"),
        "test chunk should be absent after reset"
    );
    assert!(
        setup.clip.get_file_chunks().is_empty(),
        "file_chunks should be empty after reset"
    );
}

#[test]
fn test_multiple_files_with_cross_references() -> Result<(), ChunkError> {
    // Create a temporary directory to hold input and output files
    let temp = TempDir::new()?;
    let input_dir = temp.path().join("input");
    let gen_dir = temp.path().join("gen");
    let private_dir = temp.path().join("private");

    fs::create_dir_all(&input_dir)?;
    fs::create_dir_all(&gen_dir)?;
    fs::create_dir_all(&private_dir)?;

    // Define file paths
    let file1_path = input_dir.join("file1.noweb");
    let file2_path = input_dir.join("file2.noweb");
    let file3_path = input_dir.join("file3.noweb");

    // Define content for each file
    // File 1 defines chunk_a and a file chunk <<@file file_a.txt>>=
    let file1_content = r#"
<<chunk_a>>=
Content of chunk A
@
<<@file file_a.txt>>=
File A content from file1
<<chunk_b2>>
@
"#;

    // File 2 defines chunk_b that references chunk_a and defines chunk_b2
    let file2_content = r#"
<<chunk_b>>=
Content of chunk B before referencing chunk A:
<<chunk_a>>
After referencing chunk A.
@
<<chunk_b2>>=
Content of chunk B2
@
"#;

    // File 3 defines chunk_c that references both chunk_a and chunk_b, and a file chunk <<@file file_c.txt>>=
    let file3_content = r#"
<<chunk_c>>=
Start of chunk C
<<chunk_a>>
Middle of chunk C
<<chunk_b>>
End of chunk C
@
<<@file file_c.txt>>=
File C content from file3
@
"#;

    // Write content to the files
    fs::write(&file1_path, file1_content)?;
    fs::write(&file2_path, file2_content)?;
    fs::write(&file3_path, file3_content)?;

    // Initialize SafeFileWriter with gen and private directories
    let safe_writer = SafeFileWriter::new(gen_dir.clone(), private_dir.clone());

    // Initialize Clip with default delimiters
    let mut clip = Clip::new(safe_writer, "<<", ">>", "@");

    // Read all three files
    clip.read_files(&[&file1_path, &file2_path, &file3_path])?;

    // Verify that all chunks are recognized
    assert!(clip.has_chunk("chunk_a"), "chunk_a should be present");
    assert!(clip.has_chunk("chunk_b"), "chunk_b should be present");
    assert!(clip.has_chunk("chunk_b2"), "chunk_b2 should be present");
    assert!(clip.has_chunk("chunk_c"), "chunk_c should be present");

    // Verify that file chunks are recognized
    let file_chunks = clip.get_file_chunks();
    assert_eq!(file_chunks.len(), 2, "There should be two @file chunks");

    // Check for chunk names without delimiters
    assert!(
        file_chunks.contains(&"@file file_a.txt".to_string()),
        "file_chunks should contain '@file file_a.txt'"
    );
    assert!(
        file_chunks.contains(&"@file file_c.txt".to_string()),
        "file_chunks should contain '@file file_c.txt'"
    );

    // Write file chunks to disk
    clip.write_files()?;

    // Verify the contents of the generated files
    let file_a_content = fs::read_to_string(gen_dir.join("file_a.txt"))?;
    assert_eq!(
        file_a_content.trim(),
        "File A content from file1\nContent of chunk B2",
        "file_a.txt should have correct content"
    );

    let file_c_content = fs::read_to_string(gen_dir.join("file_c.txt"))?;
    assert_eq!(
        file_c_content.trim(),
        "File C content from file3",
        "file_c.txt should have correct content"
    );

    Ok(())
}
