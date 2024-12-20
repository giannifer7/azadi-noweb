// ==== Mutual Recursion Tests ====
// Tests for detecting recursion between multiple chunks that reference each other

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
        other => panic!("Expected RecursiveReference error, got {:?}", other),
    }
}