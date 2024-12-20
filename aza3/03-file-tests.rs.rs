// ==== File Handling Tests ====
// Tests for file chunk detection, writing, and multiple file generation

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