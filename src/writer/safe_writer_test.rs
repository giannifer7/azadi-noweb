use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

use super::*;

#[test]
fn test_basic_file_writing() -> io::Result<()> {
    let temp = TempDir::new()?;
    let mut writer = SafeFileWriter::new(temp.path().join("gen"), temp.path().join("private"));

    let test_file = PathBuf::from("test.txt");
    let test_content = "Hello, World!";

    // Write initial content
    let private_path = writer.before_write(&test_file)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", test_content)?;
    }
    writer.after_write(&test_file)?;

    // Verify content
    let final_path = writer.get_gen_base().join(&test_file);
    let content = fs::read_to_string(&final_path)?;
    assert_eq!(content, test_content);

    Ok(())
}

#[test]
fn test_modification_detection() -> io::Result<()> {
    let temp = TempDir::new()?;
    let mut writer = SafeFileWriter::new(temp.path().join("gen"), temp.path().join("private"));

    let test_file = PathBuf::from("test.txt");
    let initial_content = "Initial content";
    let modified_content = "Modified content";

    // Write initial content
    let private_path = writer.before_write(&test_file)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", initial_content)?;
    }
    writer.after_write(&test_file)?;

    // Sleep briefly to ensure the modification time will be different
    thread::sleep(Duration::from_millis(10));

    // Modify the output file directly
    let final_path = writer.get_gen_base().join(&test_file);
    {
        let mut file = fs::File::create(&final_path)?;
        write!(file, "{}", modified_content)?;
    }

    // Try to write again with original content
    let private_path = writer.before_write(&test_file)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", initial_content)?;
    }
    writer.after_write(&test_file)?;

    // The modified content should remain since the file was modified after generation
    let content = fs::read_to_string(&final_path)?;
    assert_eq!(
        content, modified_content,
        "Modified content should be preserved"
    );

    Ok(())
}

#[test]
fn test_unmodified_file_update() -> io::Result<()> {
    let temp = TempDir::new()?;
    let mut writer = SafeFileWriter::new(temp.path().join("gen"), temp.path().join("private"));

    let test_file = PathBuf::from("test.txt");
    let initial_content = "Initial content";
    let new_content = "New content";

    // Write initial content
    let private_path = writer.before_write(&test_file)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", initial_content)?;
    }
    writer.after_write(&test_file)?;

    // Write new content
    let private_path = writer.before_write(&test_file)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", new_content)?;
    }
    writer.after_write(&test_file)?;

    // The new content should be written since the file wasn't modified externally
    let content = fs::read_to_string(writer.get_gen_base().join(&test_file))?;
    assert_eq!(content, new_content, "New content should be written");

    Ok(())
}

#[test]
fn test_backup_creation() -> io::Result<()> {
    let temp = TempDir::new()?;
    let mut writer = SafeFileWriter::new(temp.path().join("gen"), temp.path().join("private"));

    let test_file = PathBuf::from("test.txt");
    let content = "Test content";

    // Write content
    let private_path = writer.before_write(&test_file)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", content)?;
    }
    writer.after_write(&test_file)?;

    // Verify backup was created
    let backup_path = writer.get_old_dir().join(&test_file);
    assert!(backup_path.exists(), "Backup file should exist");

    let backup_content = fs::read_to_string(backup_path)?;
    assert_eq!(
        backup_content, content,
        "Backup content should match original"
    );

    Ok(())
}

#[test]
fn test_nested_directory_creation() -> io::Result<()> {
    let temp = TempDir::new()?;
    let mut writer = SafeFileWriter::new(temp.path().join("gen"), temp.path().join("private"));

    let nested_path = PathBuf::from("dir1/dir2/test.txt");
    let content = "Nested content";

    // Write to nested path
    let private_path = writer.before_write(&nested_path)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", content)?;
    }
    writer.after_write(&nested_path)?;

    // Verify directories were created
    let gen_dir = writer.get_gen_base().join("dir1").join("dir2");
    let old_dir = writer.get_old_dir().join("dir1").join("dir2");
    let private_dir = writer.get_private_dir().join("dir1").join("dir2");

    assert!(
        gen_dir.exists(),
        "Generated directory structure should exist"
    );
    assert!(old_dir.exists(), "Backup directory structure should exist");
    assert!(
        private_dir.exists(),
        "Private directory structure should exist"
    );

    Ok(())
}

#[test]
fn test_concurrent_modifications() -> io::Result<()> {
    let temp = TempDir::new()?;
    let mut writer = SafeFileWriter::new(temp.path().join("gen"), temp.path().join("private"));

    let test_file = PathBuf::from("test.txt");
    let initial_content = "Initial";
    let modified_content_1 = "Modified 1";
    let modified_content_2 = "Modified 2";

    // Initial write
    let private_path = writer.before_write(&test_file)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", initial_content)?;
    }
    writer.after_write(&test_file)?;

    // First modification
    thread::sleep(Duration::from_millis(10));
    let final_path = writer.get_gen_base().join(&test_file);
    {
        let mut file = fs::File::create(&final_path)?;
        write!(file, "{}", modified_content_1)?;
    }

    // Second modification
    thread::sleep(Duration::from_millis(10));
    {
        let mut file = fs::File::create(&final_path)?;
        write!(file, "{}", modified_content_2)?;
    }

    // Try to write new content
    let private_path = writer.before_write(&test_file)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", initial_content)?;
    }
    writer.after_write(&test_file)?;

    // Latest modification should be preserved
    let content = fs::read_to_string(&final_path)?;
    assert_eq!(
        content, modified_content_2,
        "Latest modification should be preserved"
    );

    Ok(())
}

#[test]
fn test_copy_if_different_with_same_content() -> io::Result<()> {
    let temp = TempDir::new()?;
    let mut writer = SafeFileWriter::new(temp.path().join("gen"), temp.path().join("private"));

    let test_file = PathBuf::from("test.txt");
    let content = "Same content";

    // Initial write
    let private_path = writer.before_write(&test_file)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", content)?;
    }
    writer.after_write(&test_file)?;

    // Get the initial modification time
    let final_path = writer.get_gen_base().join(&test_file);
    let initial_mtime = fs::metadata(&final_path)?.modified()?;

    // Write the same content again
    thread::sleep(Duration::from_millis(10));
    let private_path = writer.before_write(&test_file)?;
    {
        let mut file = fs::File::create(&private_path)?;
        write!(file, "{}", content)?;
    }
    writer.after_write(&test_file)?;

    // Verify the file wasn't touched
    let new_mtime = fs::metadata(&final_path)?.modified()?;
    assert_eq!(
        initial_mtime, new_mtime,
        "File should not be modified if content is the same"
    );

    Ok(())
}
