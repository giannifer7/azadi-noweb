# File Storage Implementation

```rust
// <[file-store]>=
#[derive(Debug, Clone)]
pub struct FileStore {
    files: Vec<String>,
}

impl FileStore {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add_file(&mut self, file: &str) -> usize {
        if let Some(pos) = self.files.iter().position(|f| f == file) {
            pos
        } else {
            let pos = self.files.len();
            self.files.push(file.to_string());
            pos
        }
    }

    pub fn get_file(&self, index: usize) -> Option<&str> {
        self.files.get(index).map(|s| s.as_str())
    }
}
// $$
```