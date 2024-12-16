# Test Infrastructure and Basic Tests

````rust
<[@file src/noweb_test.rs]>=
use std::fs;
use tempfile::TempDir;
use super::*;

<[test_setup]>

<[test_constants_basic]>

<[test_implementations_basic]>

<[test_constants_indent]>

<[test_implementations_indent]>

<[test_constants_files]>

<[test_implementations_files]>

<[test_constants_special]>

<[test_implementations_special]>

<[test_cross_references]>

<[test_files_setup]>

<[test_files_verification]>
$$


<[test_setup]>=
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
$$

<[test_constants_basic]>=
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
$$

<[test_implementations_basic]>=
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
$$
````
