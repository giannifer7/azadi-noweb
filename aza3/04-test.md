# Cross-Reference Tests

````rust
<[test_cross_references]>=
#[test]
fn test_multiple_files_with_cross_references() -> Result<(), ChunkError> {
    let temp = TempDir::new()?;
    let input_dir = temp.path().join("input");
    let gen_dir = temp.path().join("gen");
    let private_dir = temp.path().join("private");

    fs::create_dir_all(&input_dir)?;
    fs::create_dir_all(&gen_dir)?;
    fs::create_dir_all(&private_dir)?;

    <[test_files_setup]>
    <[test_files_verification]>

    Ok(())
}
$$

<[test_files_setup]>=
// Define file paths
let file1_path = input_dir.join("file1.noweb");
let file2_path = input_dir.join("file2.noweb");
let file3_path = input_dir.join("file3.noweb");

// File 1: defines chunk_a and a file chunk
let file1_content = r#"
# <<chunk_a>>=
Content of chunk A
# @
# <<@file file_a.txt>>=
File A content from file1
# <<chunk_b2>>
# @
"#;

// File 2: defines chunk_b that references chunk_a, and chunk_b2
let file2_content = r#"
# <<chunk_b>>=
Content of chunk B before referencing chunk A:
# <<chunk_a>>
After referencing chunk A.
# @
# <<chunk_b2>>=
Content of chunk B2
# @
"#;

// File 3: defines chunk_c referencing both chunk_a and chunk_b
let file3_content = r#"
# <<chunk_c>>=
Start of chunk C
# <<chunk_a>>
Middle of chunk C
# <<chunk_b>>
End of chunk C
# @
# <<@file file_c.txt>>=
File C content from file3
# @
"#;

// Write content to files
fs::write(&file1_path, file1_content)?;
fs::write(&file2_path, file2_content)?;
fs::write(&file3_path, file3_content)?;

// Initialize clip with comment markers
let safe_writer = SafeFileWriter::new(gen_dir.clone(), private_dir.clone());
let mut clip = Clip::new(safe_writer, "<<", ">>", "@", &["#".to_string()]);

// Read all three files
clip.read_files(&[&file1_path, &file2_path, &file3_path])?;
$$

<[test_files_verification]>=
// Verify that all chunks are recognized
assert!(clip.has_chunk("chunk_a"), "chunk_a should be present");
assert!(clip.has_chunk("chunk_b"), "chunk_b should be present");
assert!(clip.has_chunk("chunk_b2"), "chunk_b2 should be present");
assert!(clip.has_chunk("chunk_c"), "chunk_c should be present");

// Verify that file chunks are recognized
let file_chunks = clip.get_file_chunks();
assert_eq!(file_chunks.len(), 2, "There should be two @file chunks");

assert!(
    file_chunks.contains(&"@file file_a.txt".to_string()),
    "file_chunks should contain '@file file_a.txt'"
);
assert!(
    file_chunks.contains(&"@file file_c.txt".to_string()),
    "file_chunks should contain '@file file_c.txt'"
);

// Write file chunks to disk
clip.write_files()?;

// Verify the contents of the generated files
let file_a_content = fs::read_to_string(gen_dir.join("file_a.txt"))?;
assert_eq!(
    file_a_content.trim(),
    "File A content from file1\nContent of chunk B2",
    "file_a.txt should have correct content"
);

let file_c_content = fs::read_to_string(gen_dir.join("file_c.txt"))?;
assert_eq!(
    file_c_content.trim(),
    "File C content from file3",
    "file_c.txt should have correct content"
);
$$
````
