// ==== Test Setup and Imports ====
use super::*;
use std::fs;
use tempfile::TempDir;

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
}// ==== Test Constants and Sample Data ====
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
"#;// ==== Basic Functionality Tests ====
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
}// ==== File Operation Tests ====
#[test]
fn test_file_chunk_detection() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(FILE_CHUNKS, "test_files.nw");

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
    clip.read(file_content, "test_write_file.nw");
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

    clip.read(TWO_FILES, "test_two_files.nw");
    clip.write_files()?;

    let content1 = fs::read_to_string(gen_path.join("file1.txt"))?;
    let content2 = fs::read_to_string(gen_path.join("file2.txt"))?;

    assert_eq!(content1.trim(), "Content 1");
    assert_eq!(content2.trim(), "Content 2");
    Ok(())
}

#[test]
fn test_sequential_chunks() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(SEQUENTIAL_CHUNKS, "test_sequential.nw");

    let expanded = setup.clip.expand("main", "")?;
    assert_eq!(expanded, vec!["First part\n", "Second part\n"]);
    Ok(())
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
}// ==== Error Handling Tests ====
#[test]
fn test_undefined_chunk_error() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(
        r#"
# <<main>>=
# <<nonexistent>>
# @
"#,
        "undefined.nw",
    );

    let result = setup.clip.expand("main", "");
    match result {
        Err(AzadiError::Chunk(ChunkError::UndefinedChunk { chunk, location })) => {
            assert_eq!(chunk, "nonexistent");
            assert_eq!(location.file, "undefined.nw");
            assert_eq!(location.line, 1); // Internal line count is 0-based
        }
        _ => panic!("Expected UndefinedChunk error"),
    }
}

#[test]
fn test_recursive_chunk_error() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(
        r#"
# <<recursive>>=
Start
# <<recursive>>
End
# @
"#,
        "recursive.nw",
    );

    let result = setup.clip.expand("recursive", "");
    match result {
        Err(AzadiError::Chunk(ChunkError::RecursiveReference { chunk, location })) => {
            assert_eq!(chunk, "recursive");
            assert_eq!(location.file, "recursive.nw");
            assert_eq!(location.line, 2); // Internal line count is 0-based
        }
        _ => panic!("Expected RecursiveReference error"),
    }
}

#[test]
fn test_max_recursion_error() {
    let mut setup = TestSetup::new(&["#"]);
    
    // Create a long chain of nested chunks that exceeds MAX_DEPTH
    let mut content = String::from(r#"
# <<a-000>>=
# <<a-001>>
# @"#);

    // Generate a chain of chunks
    let chain_length = 150; // More than MAX_DEPTH = 100
    for i in 1..chain_length {
        content.push_str(&format!(r#"
# <<a-{:03}>>=
# <<a-{:03}>>
# @"#, 
            i,      // a-001, a-002, etc.
            i + 1   // a-002, a-003, etc.
        ));
    }

    setup.clip.read(&content, "max_recursion.nw");
    let result = setup.clip.expand("a-000", "").unwrap_err();
    
    assert!(matches!(
        result,
        AzadiError::Chunk(ChunkError::RecursionLimit { .. })
    ), "Expected RecursionLimit error, got {:?}", result);
}

#[test]
fn test_error_messages_format() {
    let mut setup = TestSetup::new(&["#"]);

    setup.clip.read(
        r#"
# <<a>>=
# <<nonexistent>>
# @
"#,
        "errors.nw",
    );

    let err = setup.clip.expand("a", "").unwrap_err();
    let error_msg = err.to_string();

    // Check for 1-based line numbers in error messages
    assert!(error_msg.contains("Error: errors.nw 2:"));
    assert!(error_msg.contains("referenced chunk 'nonexistent' is undefined"));
}

#[test]
fn test_error_location_in_included_chunks() {
    let mut setup = TestSetup::new(&["#"]);

    setup.clip.read(
        r#"
# <<main>>=
Before include
# <<included>>
After include
# @
"#,
        "main.nw",
    );

    setup.clip.read(
        r#"
# <<included>>=
Start of included content
# <<nonexistent>>
End of included content
# @
"#,
        "included.nw",
    );

    let result = setup.clip.expand("main", "");
    match result {
        Err(AzadiError::Chunk(ChunkError::UndefinedChunk { chunk, location })) => {
            assert_eq!(chunk, "nonexistent");
            assert_eq!(location.file, "included.nw");
            assert_eq!(location.line, 2); // Internal line count is 0-based
        }
        _ => panic!("Expected UndefinedChunk error"),
    }
}
// ==== Recursion and Mutual Recursion Tests ====
#[test]
fn test_mutual_recursion_error() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(
        r#"
# <<chunk-a>>=
Start A
# <<chunk-b>>
End A
# @

# <<chunk-b>>=
Middle B
# <<chunk-a>>
End B
# @
"#,
        "mutual_recursion.nw",
    );

    let result = setup.clip.expand("chunk-a", "");
    match result {
        Err(AzadiError::Chunk(ChunkError::RecursiveReference { chunk, location })) => {
            assert_eq!(chunk, "chunk-a");
            assert_eq!(location.file, "mutual_recursion.nw");
            assert_eq!(location.line, 8); // Line of <<chunk-a>> in chunk-b, 0-based
        }
        _ => panic!("Expected RecursiveReference error"),
    }
}

#[test]
fn test_complex_recursion() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(
        r#"
# <<a>>=
# <<b>>
# @

# <<b>>=
# <<c>>
# @

# <<c>>=
# <<d>>
# @

# <<d>>=
# <<b>>
# @
"#,
        "complex_recursion.nw",
    );

    let result = setup.clip.expand("a", "");
    match result {
        Err(AzadiError::Chunk(ChunkError::RecursiveReference { chunk, location })) => {
            assert_eq!(chunk, "b");
            assert_eq!(location.file, "complex_recursion.nw");
        }
        _ => panic!("Expected RecursiveReference error"),
    }
}

#[test]
fn test_nested_chunk_without_recursion() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(
        r#"
# <<a>>=
Start
# <<b>>
End
# @

# <<b>>=
Middle
# <<c>>
More
# @

# <<c>>=
Inner content
# @
"#,
        "nested.nw",
    );

    let result = setup.clip.expand("a", "")?;
    let expected = vec![
        "Start\n",
        "Middle\n",
        "Inner content\n",
        "More\n",
        "End\n"
    ];
    assert_eq!(result, expected, "Nested expansion without recursion should work");
    Ok(())
}

#[test]
fn test_diamond_dependency() -> Result<(), ChunkError> {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(
        r#"
# <<top>>=
# <<left>>
# <<right>>
# @

# <<left>>=
Left content
# <<bottom>>
# @

# <<right>>=
Right content
# <<bottom>>
# @

# <<bottom>>=
Bottom content
# @
"#,
        "diamond.nw",
    );

    let result = setup.clip.expand("top", "")?;
    let expected = vec![
        "Left content\n",
        "Bottom content\n",
        "Right content\n",
        "Bottom content\n"
    ];
    assert_eq!(result, expected, "Diamond dependencies should be handled correctly");
    Ok(())
}// ==== Safety and Regex Security Tests ====

#[test]
fn test_dangerous_comment_markers() {
    // These markers must be escaped before being used in the regex pattern
    let markers = &[
        "#",           // normal case
        r".*",         // regex wildcard
        r"[a-z]+",     // regex character class
        r"\d+",        // regex digit
        "<<",          // same as delimiter
        ">>",          // same as delimiter
        "(comment)",   // regex group
        "|",           // regex alternation
        r"\",          // backslash
        "*+?",         // regex quantifiers
    ];

    let content = r#"
#<<test1>>=
Content1
@
.*<<test2>>=
Content2
@
[a-z]+<<test3>>=
Content3
@
(comment)<<test4>>=
Content4
@
"#;
    
    let mut setup = TestSetup::new(markers);
    setup.clip.read(content, "regex_test.nw");

    assert!(setup.clip.has_chunk("test1"), "Basic marker # failed");
    assert!(setup.clip.has_chunk("test2"), "Wildcard marker .* failed");
    assert!(setup.clip.has_chunk("test3"), "Character class marker [a-z]+ failed");
    assert!(setup.clip.has_chunk("test4"), "Group marker (comment) failed");

    // Verify content extraction
    assert_eq!(
        setup.clip.get_chunk_content("test1").unwrap(),
        vec!["Content1\n"],
        "Content extraction failed for test1"
    );
}

#[test]
fn test_regex_dos_prevention() {
    let mut setup = TestSetup::new(&["#", "(?:[a-z]++)*"]);  // Potential catastrophic backtracking
    
    let content = r#"
# <<test>>=
Content
@
"#;
    
    setup.clip.read(content, "regex_dos.nw");
    assert!(setup.clip.has_chunk("test"), "Should handle potentially malicious regex safely");
}

#[test]
fn test_nested_delimiters() {
    let mut setup = TestSetup::new(&["#"]);
    
    // Test chunks with nested delimiters in their names
    setup.clip.read(
        r#"
# <<chunk<<nested>>>>=
Content
@

# <<chunk>>
"#,
        "nested_delims.nw"
    );

    assert!(setup.clip.has_chunk("chunk<<nested>>"), 
        "Should handle nested delimiters in chunk names");
}

#[test]
fn test_chunk_name_validation() {
    let mut setup = TestSetup::new(&["#"]);
    
    // Test potentially problematic chunk names
    let test_cases = [
        r#"# <<@file ../test.txt>>=
        Bad path
        @"#,
        
        r#"# <<@file /absolute/path.txt>>=
        Bad path
        @"#,
        
        r#"# <<@file C:\windows\path.txt>>=
        Bad path
        @"#,
        
        r#"# <<@file normal.txt>>=
        Good path
        @"#,
    ];

    for case in test_cases {
        setup.clip.read(case, "chunk_names.nw");
    }

    assert!(!setup.clip.has_chunk("@file ../test.txt"), 
        "Should reject path traversal in chunk names");
    assert!(!setup.clip.has_chunk("@file /absolute/path.txt"), 
        "Should reject absolute paths in chunk names");
    assert!(!setup.clip.has_chunk(r"@file C:\windows\path.txt"), 
        "Should reject Windows paths in chunk names");
    assert!(setup.clip.has_chunk("@file normal.txt"), 
        "Should accept normal chunk names");
}

#[test]
fn test_malformed_chunks() {
    let mut setup = TestSetup::new(&["#"]);
    
    // Test various malformed chunk definitions
    setup.clip.read(
        r#"
# <<>>=
Empty name
@

# <<incomplete>>=
No end marker

# << spaces in name >>=
Bad name
@

# <<valid>>=
Good chunk
@
"#,
        "malformed.nw"
    );

    assert!(!setup.clip.has_chunk(""), 
        "Should reject empty chunk names");
    assert!(!setup.clip.has_chunk(" spaces in name "), 
        "Should reject chunk names with spaces");
    assert!(setup.clip.has_chunk("valid"), 
        "Should accept valid chunk names");
}

#[test]
fn test_unicode_safety() {
    let mut setup = TestSetup::new(&["#", "→", "♦"]);  // Unicode comment markers
    
    setup.clip.read(
        r#"
# <<test1>>=
Content1
@

→ <<test2>>=
Content2
@

♦ <<test3>>=
Content3
@
"#,
        "unicode.nw"
    );

    assert!(setup.clip.has_chunk("test1"));
    assert!(setup.clip.has_chunk("test2"));
    assert!(setup.clip.has_chunk("test3"));
}

#[test]
fn test_comment_marker_escaping() {
    let mut setup = TestSetup::new(&["##", "#@", "@#"]);  // Markers that could interfere with chunk end marker
    
    setup.clip.read(
        r#"
## <<test1>>=
Content1
@

#@ <<test2>>=
Content2
@

@# <<test3>>=
Content3
@
"#,
        "marker_escaping.nw"
    );

    assert!(setup.clip.has_chunk("test1"));
    assert!(setup.clip.has_chunk("test2"));
    assert!(setup.clip.has_chunk("test3"));
}