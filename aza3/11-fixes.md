# Fixes for Build Issues

First, let's update the main file with proper imports:

````rust
<[@file src/main.rs]>=
use clap::Parser;
use std::fs::File;  // Changed from fs
use std::io::{self, Write};
use std::path::PathBuf;
use azadi_noweb::{AzadiError, Clip, SafeFileWriter};

<[cli_args]>

<[run_function]>=
fn run(args: Args) -> Result<(), AzadiError> {
    let comment_markers: Vec<String> = args.comment_markers
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let safe_writer = SafeFileWriter::new(&args.gen, &args.priv_dir);
    let mut clipper = Clip::new(
        safe_writer,
        &args.open_delim,
        &args.close_delim,
        &args.chunk_end,
        &comment_markers,
    );

    clipper.read_files(&args.files)?;
    clipper.write_files()?;

    if let Some(chunks) = args.chunks {
        let chunks: Vec<&str> = chunks.split(',').collect();
        if let Some(output_path) = args.output {
            let mut file = File::create(output_path)?;  // Using File directly
            clipper.get_chunk(&chunks[0], &mut file)?;
        } else {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            clipper.get_chunk(&chunks[0], &mut handle)?;
        }
    }

    Ok(())
}
$$

<[main_function]>
````

And we need to update the library to remove the unused function. In the previous implementation of noweb.rs, we should simply remove the `write_chunks` function as it's not being used anywhere.

Would you like me to:
1. Show the complete corrected files?
2. Add more error handling?
3. Add documentation for the fixed implementation?