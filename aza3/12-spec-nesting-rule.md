# Update Rules and Constraints

```rust
// <[@replace Rules and Constraints]>=
## Rules and Constraints

1. File Size and Format:
   - Each file must be smaller than 300 lines
   - Each line must be maximum 100 characters long
   - Files must be complete (no truncation or "..." ellipsis)

2. Chunk Names:
   - Must be unique across ALL files in the project
   - No spaces allowed in chunk names
   - Use hyphens or underscores instead of spaces
   - Names should be descriptive and consistent
   - Chunk and file-chunk definitions cannot be nested inside other chunks

3. File Paths:
   - Must be relative paths (no absolute paths)
   - Must not contain .. (to prevent path traversal)
   - Must use forward slashes (/) even on Windows
   - Must follow valid Rust module naming conventions when used for Rust files

4. Markdown Format:
   - All code must be within Markdown code blocks (```rust)
   - Documentation can go between code blocks
   - Code blocks must contain complete chunks
   - Each chunk must end with $$ on a separate line

5. Chunk Definition:
   - Multiple definitions of the same chunk append unless marked with @replace
   - Each chunk must have at least one definition
   - Chunks can reference other chunks
   - Each chunk definition and reference must be prefixed with // in Rust code blocks
// $$
```