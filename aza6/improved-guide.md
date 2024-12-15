# Azadi-Noweb Guide

Azadi-noweb is a literate programming tool that enables modular file generation with seamless integration into source files and markdown documentation.

## Basic Syntax

Files are written as markdown artifacts with numerical prefixes:
```
00-introduction.md
01-core_concepts.md
02-implementation.md
```

Inside each file, chunks are enclosed in markdown code blocks and prefixed with the target language's comment marker.

### Python Example

```python
# <[@file packaging/scripts/generator.py]>=
# packaging/scripts/generator.py
def main():
    # <[setup_config]>
    # <[run_generator]>

# $$

# <[setup_config]>=
config = {
    "output_dir": "build",
    "templates": "templates"
}
# $$

# <[run_generator]>=
generator = Generator(config)
generator.run()
# $$
```

### Rust Example

```rust
// <[@file src/main.rs]>=
// src/main.rs
fn main() {
    // <[init_logger]>
    // <[run_app]>
}
// $$

// <[init_logger]>=
env_logger::init();
log::info!("Starting application...");
// $$

// <[run_app]>=
let app = App::new();
app.run();
// $$
```

## Rules and Conventions

1. **Artifact Naming**:
   - Use numerical prefixes starting from 00
   - Use hyphens after numbers: `00-`, `01-`, etc.
   - Use snake_case for the rest of the name
   - Use .md extension

2. **Chunk Naming**:
   - All chunk names in snake_case
   - Must be globally unique across all artifacts
   - Start with language-appropriate comment marker
   - End with comment marker and `$$`

3. **File Chunks**:
   - Must start with `<[@file filepath]>=`
   - First line must be a comment containing the exact filepath
   - Use forward slashes (`/`) even on Windows
   - Only use relative paths, never `..`

4. **Code Blocks**:
   - Enclose chunks in markdown code blocks with appropriate language
   - Preserves syntax highlighting
   - Comment markers don't interfere with language parsers

## Complete Example

Here's a complete artifact example showing proper markdown integration:

```markdown
# Database Configuration

We first define our database settings:

```python
# <[db_config]>=
DB_CONFIG = {
    "host": "localhost",
    "port": 5432,
    "user": "admin"
}
# $$
```

Now we can use this in our main configuration file:

```python
# <[@file config/settings.py]>=
# config/settings.py
from typing import Dict

# <[db_config]>

def get_db_url() -> str:
    return f"postgresql://{DB_CONFIG['user']}@{DB_CONFIG['host']}:{DB_CONFIG['port']}"
# $$
```
```

Would you like me to:
1. Show more examples with different languages?
2. Expand on the markdown integration?
3. Add guidelines for specific use cases?