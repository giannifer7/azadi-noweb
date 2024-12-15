# Main Program

````rust
<[@file src/main.rs]>=
use azadi_noweb::{AzadiError, Clip, SafeFileWriter};
use clap::Parser;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

<[cli_args]>

<[write_chunks_fn]>

<[run_fn]>

<[main_fn]>
$$

<[cli_args]>=
#[derive(Parser)]
#[command(
    name = "azadi",
    about = "Expand chunks like noweb - A literate programming tool",
    version
)]
struct Args {
    /// Output file for --chunks [default: stdout]
    #[arg(long)]
    output: Option<PathBuf>,

    /// Names of chunks to extract (comma separated)
    #[arg(long)]
    chunks: Option<String>,

    /// Private work directory
    #[arg(long, default_value = "_azadi_work")]
    priv_dir: PathBuf,

    /// Base directory of generated files
    #[arg(long, default_value = "gen")]
    gen: PathBuf,

    /// Delimiter used to open a chunk
    #[arg(long, default_value = "<<")]
    open_delim: String,

    /// Delimiter used to close a chunk definition
    #[arg(long, default_value = ">>")]
    close_delim: String,

    /// Delimiter for chunk-end lines
    #[arg(long, default_value = "@")]
    chunk_end: String,

    /// Comment markers (comma separated)
    #[arg(long, default_value = "#,//")]
    comment_markers: String,

    /// Input files
    #[arg(required = true)]
    files: Vec<PathBuf>,
}
$$

<[write_chunks_fn]>=
fn write_chunks<W: Write>(
    clipper: &Clip,
    chunks: &[&str],
    writer: &mut W,
) -> Result<(), AzadiError> {
    for chunk in chunks {
        clipper.get_chunk(chunk, writer)?;
        writeln!(writer)?;
    }
    Ok(())
}
$$

<[run_fn]>=
fn run(args: Args) -> Result<(), AzadiError> {
    let comment_markers: Vec<String> = args
        .comment_markers
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
            let mut file = File::create(output_path)?;
            write_chunks(&clipper, &chunks, &mut file)?;
        } else {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            write_chunks(&clipper, &chunks, &mut handle)?;
        }
    }

    Ok(())
}
$$

<[main_fn]>=
fn main() {
    let args = Args::parse();

    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
$$
````

Would you like me to show:
1. The corrected noweb.rs with ChunkWriter in its proper place?
2. Updates to any other files affected by this change?
3. New tests to verify the corrected structure?