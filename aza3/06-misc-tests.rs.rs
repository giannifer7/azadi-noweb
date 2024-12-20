// ==== Miscellaneous Tests ====
// Tests for sequential chunks, empty chunks, and chunk reset functionality

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
}

#[test]
fn test_reset() {
    let mut setup = TestSetup::new(&["#"]);

    setup.clip.read(
        r#"
# <<test>>=
Hello
# @
"#,
        "test_reset.nw",
    );

    assert!(setup.clip.has_chunk("test"));

    setup.clip.reset();

    assert!(!setup.clip.has_chunk("test"));
    assert!(setup.clip.get_file_chunks().is_empty());
}

// ==== Warnings Tests ====
// Tests for unused chunk detection and warning messages

#[test]
fn test_unused_chunk_warning() {
    let mut setup = TestSetup::new(&["#"]);

    setup.clip.read(
        r#"
# <<main>>=
# <<used1>>
# <<used2>>
# @

# <<used1>>=
Content 1
# @

# <<used2>>=
Content 2
# @

# <<unused1>>=
Never used content 1
# @

# <<unused2>>=
Never used content 2
# @

# <<@file output.txt>>=
Some file content
# @
"#,
        "unused_chunks.nw",
    );

    // First expand a chunk to trigger reference counting
    let _ = setup.clip.expand("main", "");

    // Get warnings about unused chunks
    let warnings = setup.clip.check_unused_chunks();

    // We expect exactly two warnings for unused1 and unused2
    assert_eq!(warnings.len(), 2, "Expected exactly two warnings");

    // Verify warning format and content
    for warning in warnings {
        assert!(
            warning.starts_with("Warning: unused_chunks.nw"),
            "Warning should start with file name"
        );
        assert!(
            warning.contains("chunk 'unused1' is defined but never referenced")
                || warning.contains("chunk 'unused2' is defined but never referenced"),
            "Warning should mention the unused chunk: {}",
            warning
        );
    }
}
