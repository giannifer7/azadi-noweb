# Azadi Noweb

*Azadi Noweb* helps you organize code into labeled sections called "chunks." You can mark these chunks using customizable symbols (like `<<` and `>>`). The tool can then extract specific chunks and either save them as separate files or display them directly. While it draws inspiration from noweb's literate programming approach, it's designed to be a simpler, more focused tool.

## Features

- Parses chunks defined by a starting delimiter and an ending line containing only a chunk-end marker
- Supports extracting specific named chunks and printing them to stdout or writing them to a file
- Supports "file chunks" (chunks whose names start with @file) that are automatically written to a gen directory
- Allows custom delimiters for chunk definitions and chunk endings
- Can indent nested chunks as they are expanded

## Default Delimiters

- **Open Delimiter:** `<<`
- **Close Delimiter:** `>>`
- **Chunk-End Delimiter:** `@`

These can be changed via command-line arguments (see below).

## Basic Usage

```bash
azadi-noweb [OPTIONS] <files>...
```

### Common Options

- `--open-delim`: Delimiter used to open a chunk (default: `<<`)
- `--close-delim`: Delimiter used to close a chunk definition (default: `>>`)
- `--chunk-end`: Delimiter for chunk-end lines (default: `@`)
- `--chunks`: Comma-separated list of chunk names to extract
- `--output`: Output file for extracted chunks (defaults to stdout)
- `--priv-dir`: Private work directory for temporary files (default: `_azadi_work`)
- `--gen`: Base directory where generated files are written (default: `gen`)

### Default Behavior

When you run:

```bash
azadi-noweb input.nw
```

It:

1. Reads `input.nw` and finds chunks defined like:

```azadi-noweb
<<chunkname>>=
Some code here
@
```

This defines a chunk called `chunkname` whose content is `Some code here\n`.

2. If the chunk name starts with `@file`, for example:

```azadi-noweb
<<@file output.txt>>=
File content
@
```

Azadi Noweb will write this content to `gen/output.txt`.

3. If `--chunks` is not specified, Azadi Noweb just processes file chunks and writes them into the `gen` directory. Non-file chunks are stored internally but not printed.

### Extracting Specific Chunks

If you have:

```azadi-noweb
<<test>>=
Hello
@
```

Running:

```bash
azadi-noweb --chunks test input.nw
```

will print:

```azadi-noweb
Hello
```

to stdout.

To save it to a file instead:

```bash
azadi-noweb --chunks test --output extracted.txt input.nw
```

This writes `Hello\n` into `extracted.txt`.

### Nested Chunks

Chunks can reference other chunks by including them inline. For example:

```azadi-noweb
<<outer>>=
Before
<<inner>>
After
@

<<inner>>=
Nested content
@
```

When you expand `outer`, it inlines `inner`. Expanding `outer` results in:

```azadi-noweb
Before
Nested content
After
```

### File Chunks

When a chunk name starts with `@file`, Azadi Noweb writes it automatically to the `gen` directory. For security reasons, the file path:
- Must be relative (cannot start with `/` or drive letters like `C:/`)
- Cannot contain `..` components
- Must use forward slashes (`/`) as path delimiters, even on Windows
- Cannot be absolute Windows paths (e.g., `C:/foo/bar.txt` is forbidden)

These restrictions ensure all generated files stay within the `gen` directory, preventing potential security risks from path traversal.

```azadi-noweb
# These are allowed:
<<@file src/config.json>>=
{
    "port": 8080
}
@

<<@file nested/deep/file.txt>>=
content
@

# These are NOT allowed and will result in an error:
<<@file ../outside.txt>>=
content
@

<<@file /etc/passwd>>=
content
@

<<@file C:/Windows/System32/config.txt>>=
content
@

<<@file nested\deep\file.txt>>=  # Wrong path delimiter
content
@
```

After running:

```bash
azadi-noweb input.nw
```

You'll find the allowed files at `gen/src/config.json` and `gen/nested/deep/file.txt` respectively. The attempts to write outside the `gen` directory will be rejected with an error.

Multiple file chunks are also supported:

```azadi-noweb
<<@file file1.txt>>=
Content 1
@

<<@file file2.txt>>=
Content 2
@
```

Both `file1.txt` and `file2.txt` will appear in `gen/` after processing.

### Indentation Handling

If a chunk references another chunk with indentation, the indentation is preserved:

```azadi-noweb
<<main>>=
    <<indented>>
@

<<indented>>=
some code
@
```

Expanding `main` yields:

```azadi-noweb
    some code
```

Notice how the indentation before `<<indented>>` was applied to its content.

### Recursive Chunks

If a chunk references itself, directly or mutually, Azadi Noweb detects this and returns an error:

```azadi-noweb
<<recursive>>=
Start
<<recursive>>
End
@
```

Expanding `recursive` results in a `RecursiveReference` error.

### Multi-File Example

Azadi Noweb can process multiple input files at once. This is useful when you want to split your chunks across different files for better organization. For example:

```azadi-noweb
# config.nw
<<@file config.json>>=
{
    "port": 8080,
    "host": "localhost"
}
@

# server.nw
<<@file server.js>>=
const config = require('./config.json');
<<setup-server>>
@

<<setup-server>>=
const server = http.createServer((req, res) => {
    res.writeHead(200);
    res.end('Hello World');
});
server.listen(config.port, config.host);
@
```

You can process both files together:

```bash
azadi-noweb config.nw server.nw
```

This will:
1. Process all files in order
2. Generate `gen/config.json` and `gen/server.js`
3. Use any chunks defined in earlier files when processing later files

You can also extract specific chunks from multiple files:

```bash
azadi-noweb --chunks setup-server config.nw server.nw
```

### Custom Delimiters

If you want to use different delimiters, for example `<[` and `]>` for chunk definitions and `%` for chunk ends:

```bash
azadi-noweb --open-delim '<[' --close-delim ']>' --chunk-end '%' example.nw
```

Make sure the input file uses these delimiters accordingly:

```azadi-noweb
<[test]>=
Some code
%
```

### Contributing

Contributions are welcome. Please open an issue or submit a pull request if you have improvements or find bugs.

### License

This project is licensed under the terms of the MIT license.
