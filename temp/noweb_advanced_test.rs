mod test_common;
use test_common::*;
use tempfile::TempDir;

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

    assert!(error_msg.contains("Error: errors.nw 2:"));
    assert!(error_msg.contains("referenced chunk 'nonexistent' is undefined"));
}
#[test]
fn test_direct_recursion() {
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
fn test_mutual_recursion() {
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
fn test_max_recursion_depth() {
    let mut setup = TestSetup::new(&["#"]);
    
    // Create a long chain of nested chunks that exceeds MAX_DEPTH
    let mut content = String::from(r#"
# <<a-000>>=
# <<a-001>>
# @"#);

    // Generate enough chunks to exceed the limit
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
    let result = setup.clip.expand("a-000", "");
    
    assert!(matches!(
        result,
        Err(AzadiError::Chunk(ChunkError::RecursionLimit { .. }))
    ), "Expected RecursionLimit error");
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
}
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

    // Verify content extraction still works
    assert_eq!(
        setup.clip.get_chunk_content("test1").unwrap(),
        vec!["Content1\n"]
    );
}

#[test]
fn test_path_safety() {
    let mut setup = TestSetup::new(&["#"]);
    
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
        "Should reject path traversal");
    assert!(!setup.clip.has_chunk("@file /absolute/path.txt"), 
        "Should reject absolute paths");
    assert!(!setup.clip.has_chunk(r"@file C:\windows\path.txt"), 
        "Should reject Windows paths");
    assert!(setup.clip.has_chunk("@file normal.txt"), 
        "Should accept normal paths");
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

    // Verify content is correctly preserved
    assert_eq!(
        setup.clip.get_chunk_content("test1").unwrap(),
        vec!["Content1\n"]
    );
}
