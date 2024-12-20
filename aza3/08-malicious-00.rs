// Tests to verify safety against malicious input
#[test]
fn test_nested_delimiters() {
    let mut setup = TestSetup::new(&["#"]);

    // Try to confuse the parser with nested delimiters
    setup.clip.read(
        r#"
# <<outer<<inner>>>>=
Content
@
"#,
        "nested_delims.nw",
    );

    // The chunk name should be parsed as "outer<<inner"
    assert!(!setup.clip.has_chunk("outer"));
    assert!(!setup.clip.has_chunk("inner"));
    assert!(setup.clip.has_chunk("outer<<inner>>"));
}
