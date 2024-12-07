use chrono::{DateTime, Local};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Manages safe file writing with backup functionality and modification detection.
pub struct SafeFileWriter {
    gen_base: PathBuf,
    private_dir: PathBuf,
    old_dir: PathBuf,
    old_timestamp: Option<DateTime<Local>>,
}

impl SafeFileWriter {
    /// Initialize the SafeFileWriter with base and private directories.
    ///
    /// # Arguments
    /// * `gen_base` - Base directory for generated files
    /// * `private_dir` - Directory for private/temporary files
    pub fn new<P: AsRef<Path>>(gen_base: P, private_dir: P) -> Self {
        let gen_base = gen_base.as_ref().to_path_buf();
        let private_dir = private_dir.as_ref().to_path_buf();
        let old_dir = private_dir.join("__old__");

        // Create all required directories
        fs::create_dir_all(&gen_base).expect("Failed to create gen_base directory");
        fs::create_dir_all(&private_dir).expect("Failed to create private directory");
        fs::create_dir_all(&old_dir).expect("Failed to create old directory");

        SafeFileWriter {
            gen_base,
            private_dir,
            old_dir,
            old_timestamp: None,
        }
    }

    /// Copy a file only if the content differs from the destination.
    ///
    /// # Arguments
    /// * `source` - Source file path
    /// * `destination` - Destination file path
    fn copy_if_different<P: AsRef<Path>>(&self, source: P, destination: P) -> io::Result<()> {
        let source = source.as_ref();
        let destination = destination.as_ref();

        if !destination.exists() {
            return fs::copy(source, destination).map(|_| ());
        }

        let mut source_content = String::new();
        let mut dest_content = String::new();

        fs::File::open(source)?.read_to_string(&mut source_content)?;
        fs::File::open(destination)?.read_to_string(&mut dest_content)?;

        if source_content != dest_content {
            println!("file {} changed", destination.display());
            fs::copy(source, destination)?;
        }

        Ok(())
    }

    /// Prepare directories for writing a file and validate the path.
    ///
    /// # Arguments
    /// * `file_path` - Path of file to be written
    fn prepare_write_file<P: AsRef<Path>>(&self, file_path: P) -> io::Result<PathBuf> {
        let path = file_path.as_ref();
        let dest_dir = path.parent().unwrap_or_else(|| Path::new(""));

        // Create all necessary directories
        fs::create_dir_all(self.gen_base.join(dest_dir))?;
        fs::create_dir_all(self.old_dir.join(dest_dir))?;
        fs::create_dir_all(self.private_dir.join(dest_dir))?;

        Ok(path.to_path_buf())
    }

    /// Prepare for writing a file by setting up paths and checking timestamps.
    ///
    /// # Arguments
    /// * `file_name` - Name of file to write
    ///
    /// # Returns
    /// Path to write the private file to
    pub fn before_write<P: AsRef<Path>>(&mut self, file_name: P) -> io::Result<PathBuf> {
        let path = self.prepare_write_file(&file_name)?;

        let old_file_name = self.old_dir.join(&path);
        if old_file_name.is_file() {
            let metadata = fs::metadata(&old_file_name)?;
            let system_time: SystemTime = metadata.modified()?;
            self.old_timestamp = Some(DateTime::from(system_time));
        } else {
            self.old_timestamp = None;
        }

        Ok(self.private_dir.join(path))
    }

    /// Complete the file writing process by managing backups and checking modifications.
    ///
    /// # Arguments
    /// * `file_name` - Name of the file that was written
    pub fn after_write<P: AsRef<Path>>(&self, file_name: P) -> io::Result<()> {
        let path = self.prepare_write_file(file_name)?;

        let private_file = self.private_dir.join(&path);
        let output_file = self.gen_base.join(&path);
        let old_file = self.old_dir.join(&path);

        // Copy to old file first
        fs::copy(&private_file, &old_file)?;

        if output_file.is_file() {
            let system_time: SystemTime = fs::metadata(&output_file)?.modified()?;
            let out_timestamp: DateTime<Local> = DateTime::from(system_time);

            if let Some(old_timestamp) = self.old_timestamp {
                if out_timestamp > old_timestamp {
                    println!(
                        "{} modified after the last generation",
                        output_file.display()
                    );
                    // Don't overwrite the file if it was modified
                    return Ok(());
                }
            }
        }

        self.copy_if_different(&private_file, &output_file)?;

        Ok(())
    }

    #[cfg(test)]
    pub fn get_gen_base(&self) -> &Path {
        &self.gen_base
    }

    #[cfg(test)]
    pub fn get_old_dir(&self) -> &Path {
        &self.old_dir
    }

    #[cfg(test)]
    pub fn get_private_dir(&self) -> &Path {
        &self.private_dir
    }
}
