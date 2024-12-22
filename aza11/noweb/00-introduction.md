# chunkstore.nw

This document describes the implementation of the ChunkStore system for literate programming, with support for the `@replace` directive to allow chunk redefinition.

## Overview

The ChunkStore system manages and processes chunks of code, supporting expansion, recursion detection, and file operations. Each chunk is a named piece of code or text that can reference other chunks for expansion.

Key features:
- Parse input according to configurable delimiters and comment markers
- Track chunk locations for error reporting
- Handle chunk expansion with recursion and indentation control
- Support file output with path safety checks
- Allow chunk redefinition with `@replace`

## File Structure

Here's the main structure of our implementation:

```rust
// <[@file src/noweb.rs]>=
// src/noweb.rs
// <[imports]>
// <[types-and-errors]>
// <[chunk-store]>
// <[chunk-writer]>
// <[clip]>
// $$
```

## Base Dependencies

The imports we need for our implementation:

```rust
// <[imports]>=
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, Component};

use crate::AzadiError;
use crate::SafeFileWriter;
use crate::SafeWriterError;
// $$
```
