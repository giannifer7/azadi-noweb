# Special Cases and Cross-Reference Tests

````rust
<[test_constants_special]>=
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
$$

<[test_implementations_special]>=
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
$$
````
