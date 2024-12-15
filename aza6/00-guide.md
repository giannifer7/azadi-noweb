# Azadi-Noweb Guide

Azadi-noweb is a literate programming tool that enables modular file generation with seamless integration into source files and markdown documentation.

## Chunk Expressions

Every chunk expression consists of three parts:
1. A comment marker appropriate for the target language (e.g., `#` for Python, `//` for Rust)
2. The chunk name enclosed in `<[` and `]>` with optional `@file` for file chunks
3. An equals sign `=` for chunk definitions or no equals sign for chunk usage

For example in Python:
\```python
# <[my_chunk]>=     # Definition of a regular chunk
# <[my_chunk]>      # Usage of a chunk
# <[@file path]>=   # Definition of a file chunk
\```

## Chunk Names

1. Regular chunk names:
   - Must be in snake_case
   - Must be globally unique
   - Cannot contain `@file`

2. File chunk names:
   - Must start with `@file`
   - Followed by a space and the relative filepath
   - The filepath must not contain `..`
   - Use forward slashes (`/`) even on Windows

## Rules and Conventions

1. **Artifact Naming**:
   - Use numerical prefixes starting from 00
   - Use hyphens after numbers: `00-`, `01-`, etc.
   - Use snake_case for the rest of the name
   - Use .md extension

2. **Code Blocks**:
   - Enclose chunks in markdown code blocks with appropriate language
   - Each chunk definition starts with a comment marker
   - Each chunk definition ends with a comment marker and `$$`

## Language Examples

### Python Example

\```python
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
\```

### Rust Example

\```rust
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
\```

## Complete Documentation Example

Here's how to write documentation that includes chunk definitions:

Here we define our database configuration:

\```python
# <[db_config]>=
DB_CONFIG = {
    "host": "localhost",
    "port": 5432,
    "user": "admin"
}
# $$
\```

And here's how we use it in our settings file:

\```python
# <[@file config/settings.py]>=
# config/settings.py
from typing import Dict

# <[db_config]>

def get_db_url() -> str:
    return f"postgresql://{DB_CONFIG['user']}@{DB_CONFIG['host']}:{DB_CONFIG['port']}"
# $$
\```

The documentation remains readable in both plain text and rendered markdown, while the chunks can be processed by azadi-noweb to generate the actual source files.
