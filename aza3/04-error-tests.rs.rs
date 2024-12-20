// ==== Error Handling Tests ====
// Tests for various error conditions including undefined chunks,
// recursive references, and maximum recursion depth

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