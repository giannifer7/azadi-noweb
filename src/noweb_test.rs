use std::fs;
use tempfile::TempDir;
use super::*;

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

const BASIC_CHUNK: &str = r#"
# <<test>>=
Hello
# @
"#;

const TWO_CHUNKS: &str = r#"
# <<chunk1>>=
First chunk
# @
# <<chunk2>>=
Second chunk
# @
"#;

const NESTED_CHUNKS: &str = r#"
# <<outer>>=
Before
# <<inner>>
After
# @
# <<inner>>=
Nested content
# @
"#;

#[test]
fn test_basic_chunk() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(BASIC_CHUNK);

    assert!(setup.clip.has_chunk("test"));
    assert_eq!(
        setup.clip.get_chunk_content("test").unwrap(),
        vec!["Hello\n"]
    );
}

#[test]
fn test_multiple_chunks() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(TWO_CHUNKS);

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
    setup.clip.read(NESTED_CHUNKS);

    let expanded = setup.clip.expand("outer", "")?;
    let expected = vec!["Before\n", "Nested content\n", "After\n"];
    assert_eq!(expanded, expected, "Nested chunks should expand correctly");
    Ok(())
}

const INDENTED_CHUNK: &str = r#"
# <<main>>=
    # <<indented>>
# @
# <<indented>>=
some code
# @
"#;

const PYTHON_CODE: &str = r#"
# <<code>>=
def example():
    # <<body>>
# @
# <<body>>=
print('hello')
# @
"#;

const MULTI_COMMENT_CHUNKS: &str = r#"
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

#[test]
fn test_indentation_preservation() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new(&["#"]);
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
fn test_complex_indentation() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(PYTHON_CODE);

    let expanded = setup.clip.expand("code", "")?;
    let expected = vec!["def example():\n", "    print('hello')\n"];
    assert_eq!(expanded, expected);

    // Test with additional base indentation
    let expanded_indented = setup.clip.expand("code", "  ")?;
    let expected_indented = vec!["  def example():\n", "      print('hello')\n"];
    assert_eq!(expanded_indented, expected_indented);
    Ok(())
}

#[test]
fn test_multi_comment_styles() {
    let mut setup = TestSetup::new(&["#", "//"]);
    setup.clip.read(MULTI_COMMENT_CHUNKS);

    assert!(setup.clip.has_chunk("python_chunk"));
    assert!(setup.clip.has_chunk("rust_chunk"));

    let python_content = setup.clip.get_chunk_content("python_chunk").unwrap();
    assert!(python_content.join("").contains("print(\"Hello\")"));

    let rust_content = setup.clip.get_chunk_content("rust_chunk").unwrap();
    assert!(rust_content.join("").contains("println!(\"Hello\")"));
}

const FILE_CHUNKS: &str = r#"
# <<@file output.txt>>=
content
# @
# <<other>>=
other content
# @
"#;

const TWO_FILES: &str = r#"
# <<@file file1.txt>>=
Content 1
# @
# <<@file file2.txt>>=
Content 2
# @
"#;

#[test]
fn test_file_chunk_detection() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(FILE_CHUNKS);

    let file_chunks = setup.clip.get_file_chunks();
    assert_eq!(file_chunks.len(), 1);
    assert!(file_chunks.contains(&"@file output.txt".to_string()));
}

#[test]
fn test_file_writing() -> Result<(), ChunkError> {
    let temp = TempDir::new()?;
    let gen_path = temp.path().join("gen");
    let private_path = temp.path().join("private");
    fs::create_dir_all(&gen_path)?;
    fs::create_dir_all(&private_path)?;
    let safe_writer = SafeFileWriter::new(gen_path.clone(), private_path);

    let mut clip = Clip::new(safe_writer, "<<", ">>", "@", &["#".to_string()]);

    let file_content = r#"
# <<@file test.txt>>=
Hello, World!
# @
"#;
    clip.read(file_content);
    assert!(clip.has_chunk("@file test.txt"));

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
    fs::create_dir_all(&gen_path)?;
    fs::create_dir_all(&private_path)?;
    let safe_writer = SafeFileWriter::new(gen_path.clone(), private_path);

    let mut clip = Clip::new(safe_writer, "<<", ">>", "@", &["#".to_string()]);

    clip.read(TWO_FILES);
    clip.write_files()?;

    let content1 = fs::read_to_string(gen_path.join("file1.txt"))?;
    let content2 = fs::read_to_string(gen_path.join("file2.txt"))?;

    assert_eq!(content1.trim(), "Content 1");
    assert_eq!(content2.trim(), "Content 2");
    Ok(())
}

const SEQUENTIAL_CHUNKS: &str = r#"
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

const EMPTY_CHUNK: &str = r#"
# <<empty>>=
# @
"#;

#[test]
fn test_sequential_chunks() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(SEQUENTIAL_CHUNKS);

    let expanded = setup.clip.expand("main", "")?;
    assert_eq!(
        expanded,
        vec!["First part\n", "Second part\n"]
    );
    Ok(())
}

#[test]
fn test_empty_chunk() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(EMPTY_CHUNK);

    assert!(setup.clip.has_chunk("empty"));
    assert!(
        setup.clip.get_chunk_content("empty").unwrap().is_empty(),
        "empty chunk should have no content"
    );
}

#[test]
fn test_recursive_chunk() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(r#"
# <<recursive>>=
Start
# <<recursive>>
End
# @
"#);

    let result = setup.clip.expand("recursive", "");
    assert!(matches!(
        result,
        Err(AzadiError::Chunk(ChunkError::RecursiveReference(_)))
    ));
}

#[test]
fn test_reset() {
    let mut setup = TestSetup::new(&["#"]);

    setup.clip.read(r#"
# <<test>>=
Hello
# @
"#);

    assert!(setup.clip.has_chunk("test"));

    setup.clip.reset();

    assert!(!setup.clip.has_chunk("test"));
    assert!(setup.clip.get_file_chunks().is_empty());
}

#[test]
fn test_multiple_files_with_cross_references() -> Result<(), ChunkError> {
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

    // File 1: defines chunk_a and a file chunk
    let file1_content = r#"
    # <<chunk_a>>=
    Content of chunk A
    # @
    # <<@file file_a.txt>>=
    File A content from file1
    # <<chunk_b2>>
    # @
    "#;

    // File 2: defines chunk_b that references chunk_a, and chunk_b2
    let file2_content = r#"
    # <<chunk_b>>=
    Content of chunk B before referencing chunk A:
    # <<chunk_a>>
    After referencing chunk A.
    # @
    # <<chunk_b2>>=
    Content of chunk B2
    # @
    "#;

    // File 3: defines chunk_c referencing both chunk_a and chunk_b
    let file3_content = r#"
    # <<chunk_c>>=
    Start of chunk C
    # <<chunk_a>>
    Middle of chunk C
    # <<chunk_b>>
    End of chunk C
    # @
    # <<@file file_c.txt>>=
    File C content from file3
    # @
    "#;

    // Write content to files
    fs::write(&file1_path, file1_content)?;
    fs::write(&file2_path, file2_content)?;
    fs::write(&file3_path, file3_content)?;

    // Initialize clip with comment markers
    let safe_writer = SafeFileWriter::new(gen_dir.clone(), private_dir.clone());
    let mut clip = Clip::new(safe_writer, "<<", ">>", "@", &["#".to_string()]);

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

// Define file paths
const file1_path: &str = input_dir.join("file1.noweb");
let file2_path = input_dir.join("file2.noweb");
let file3_path = input_dir.join("file3.noweb");

// File 1: defines chunk_a and a file chunk
let file1_content = r#"
# <<chunk_a>>=
Content of chunk A
# @
# <<@file file_a.txt>>=
File A content from file1
# <<chunk_b2>>
# @
"#;

// File 2: defines chunk_b that references chunk_a, and chunk_b2
let file2_content = r#"
# <<chunk_b>>=
Content of chunk B before referencing chunk A:
# <<chunk_a>>
After referencing chunk A.
# @
# <<chunk_b2>>=
Content of chunk B2
# @
"#;

// File 3: defines chunk_c referencing both chunk_a and chunk_b
let file3_content = r#"
# <<chunk_c>>=
Start of chunk C
# <<chunk_a>>
Middle of chunk C
# <<chunk_b>>
End of chunk C
# @
# <<@file file_c.txt>>=
File C content from file3
# @
"#;

// Write content to files
fs::write(&file1_path, file1_content)?;
fs::write(&file2_path, file2_content)?;
fs::write(&file3_path, file3_content)?;

// Initialize clip with comment markers
let safe_writer = SafeFileWriter::new(gen_dir.clone(), private_dir.clone());
let mut clip = Clip::new(safe_writer, "<<", ">>", "@", &["#".to_string()]);

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
