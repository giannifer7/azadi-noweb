// ==== Regex Safety Tests ====
// Tests to verify safety against malicious regex patterns in markers and delimiters

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
    
    // This will fail until we fix the regex pattern construction
    // to properly escape the comment markers
    let mut setup = TestSetup::new(markers);
    setup.clip.read(content, "regex_test.nw");

    // These assertions should pass once comment markers are properly escaped
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