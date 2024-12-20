// Test Setup and Imports
// Contains the TestSetup struct and basic imports needed for testing

use super::*;
use std::fs;
use tempfile::TempDir;

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

        let comment_markers = comment_markers
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let clip = Clip::new(safe_writer, "<<", ">>", "@", &comment_markers);

        TestSetup {
            _temp_dir: temp_dir,
            clip,
        }
    }
}