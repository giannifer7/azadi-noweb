use std::fs;
use std::io;
use tempfile::TempDir;

use super::*;

struct TestSetup {
    _temp_dir: TempDir,
    clip: Clip,
}

impl TestSetup {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let safe_writer =
            SafeFileWriter::new(temp_dir.path().join("gen"), temp_dir.path().join("private"));
        let clip = Clip::new(safe_writer);

        TestSetup {
            _temp_dir: temp_dir,
            clip,
        }
    }
}

#[test]
fn test_basic_chunk_recognition() {
    let mut setup = TestSetup::new();

    setup.clip.read(
        "\
<<test>>=
Hello
@
",
    );

    assert!(
        setup.clip.has_chunk("test"),
        "Should recognize chunk named 'test'"
    );
    assert_eq!(
        setup.clip.get_chunk_content("test").unwrap(),
        &vec!["Hello\n"]
    );
}

#[test]
fn test_multiple_chunks() {
    let mut setup = TestSetup::new();

    setup.clip.read(
        "\
<<chunk1>>=
First chunk
@

<<chunk2>>=
Second chunk
@
",
    );

    assert!(setup.clip.has_chunk("chunk1"));
    assert!(setup.clip.has_chunk("chunk2"));
    assert_eq!(
        setup.clip.get_chunk_content("chunk1").unwrap(),
        &vec!["First chunk\n"]
    );
    assert_eq!(
        setup.clip.get_chunk_content("chunk2").unwrap(),
        &vec!["Second chunk\n"]
    );
}

#[test]
fn test_nested_chunk_expansion() {
    let mut setup = TestSetup::new();

    setup.clip.read(
        "\
<<outer>>=
Before
<<inner>>
After
@
<<inner>>=
Nested content
@
",
    );

    let expanded = setup.clip.expand("outer", "");
    let expected = vec!["Before\n", "Nested content\n", "After\n"];
    assert_eq!(expanded, expected);
}

#[test]
fn test_indentation_preservation() {
    let mut setup = TestSetup::new();

    setup.clip.read(
        "\
<<main>>=
    <<indented>>
@
<<indented>>=
some code
@
",
    );

    let expanded = setup.clip.expand("main", "");
    assert_eq!(expanded, vec!["    some code\n"]);
}

#[test]
fn test_file_chunk_detection() {
    let mut setup = TestSetup::new();

    setup.clip.read(
        "\
<<@file output.txt>>=
content
@
<<other>>=
other content
@
",
    );

    let file_chunks = setup.clip.get_file_chunks();
    assert_eq!(file_chunks.len(), 1);
    assert!(file_chunks.contains(&"@file output.txt".to_string()));
}

#[test]
fn test_chunk_output_to_writer() -> io::Result<()> {
    let mut setup = TestSetup::new();
    let mut output = Vec::new();

    setup.clip.read(
        "\
<<test>>=
Line 1
Line 2
@
",
    );
    setup.clip.get_chunk("test", &mut output)?;

    let result = String::from_utf8(output).unwrap();
    assert_eq!(result, "Line 1\nLine 2\n\n");
    Ok(())
}

#[test]
fn test_multiple_chunk_references() {
    let mut setup = TestSetup::new();

    setup.clip.read(
        "\
<<main>>=
<<part1>>
<<part2>>
@
<<part1>>=
First part
@
<<part2>>=
Second part
@
",
    );

    let expanded = setup.clip.expand("main", "");
    let expected = vec!["First part\n", "Second part\n"];
    assert_eq!(expanded, expected);
}

#[test]
fn test_file_writing() -> io::Result<()> {
    let temp = TempDir::new()?;
    let safe_writer = SafeFileWriter::new(temp.path().join("gen"), temp.path().join("private"));
    let mut clip = Clip::new(safe_writer);

    clip.read(
        "\
<<@file test.txt>>=
Hello, World!
@
",
    );

    clip.write_files()?;

    let content = fs::read_to_string(temp.path().join("gen/test.txt"))?;
    assert_eq!(content.trim(), "Hello, World!");
    Ok(())
}

#[test]
fn test_multiple_file_generation() -> io::Result<()> {
    let temp = TempDir::new()?;
    let safe_writer = SafeFileWriter::new(temp.path().join("gen"), temp.path().join("private"));
    let mut clip = Clip::new(safe_writer);

    clip.read(
        "\
<<@file file1.txt>>=
Content 1
@
<<@file file2.txt>>=
Content 2
@
",
    );

    clip.write_files()?;

    let content1 = fs::read_to_string(temp.path().join("gen/file1.txt"))?;
    let content2 = fs::read_to_string(temp.path().join("gen/file2.txt"))?;

    assert_eq!(content1.trim(), "Content 1");
    assert_eq!(content2.trim(), "Content 2");
    Ok(())
}

#[test]
fn test_empty_chunk() {
    let mut setup = TestSetup::new();

    setup.clip.read(
        "\
<<empty>>=
@
",
    );

    assert!(setup.clip.has_chunk("empty"));
    assert!(setup.clip.get_chunk_content("empty").unwrap().is_empty());
}

#[test]
fn test_complex_indentation() {
    let mut setup = TestSetup::new();

    setup.clip.read(
        "\
<<code>>=
def example():
    <<body>>
@
<<body>>=
print('hello')
@
",
    );

    let expanded = setup.clip.expand("code", "");
    let expected = vec!["def example():\n", "    print('hello')\n"];
    assert_eq!(
        expanded, expected,
        "Indentation should be preserved from the chunk reference"
    );

    // Also test with additional base indentation
    let expanded_indented = setup.clip.expand("code", "  ");
    let expected_indented = vec![
        "  def example():\n",
        "      print('hello')\n", // Should have both the base indent and the nested indent
    ];
    assert_eq!(
        expanded_indented, expected_indented,
        "Both base and nested indentation should be preserved"
    );
}
