use clap::Parser;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use azadi_noweb::{AzadiError, Clip, SafeFileWriter};

#[derive(Parser)]
#[command(
    name = "azadi",
    about = "Expand chunks like noweb - A literate programming tool",
    version
)]
struct Args {
    /// Output file for --chunks (defaults to stdout)
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

    /// Delimiter used to open a chunk (default: "<<")
    #[arg(long, default_value = "<<")]
    open_delim: String,

    /// Delimiter used to close a chunk definition (default: ">>")
    #[arg(long, default_value = ">>")]
    close_delim: String,

    /// Delimiter for chunk-end lines (default: "@")
    #[arg(long, default_value = "@")]
    chunk_end: String,

    /// Input files
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

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

fn run(args: Args) -> Result<(), AzadiError> {
    // Create the SafeFileWriter using default configuration
    let safe_writer = SafeFileWriter::new(&args.gen, &args.priv_dir);

    // Create the Clip with the specified or default delimiters
    let mut clipper = Clip::new(
        safe_writer,
        &args.open_delim,
        &args.close_delim,
        &args.chunk_end,
    );

    clipper.read_files(&args.files)?;

    // Write all file chunks to their respective files
    clipper.write_files()?;

    // If there are chunks to extract to stdout or a file
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

fn main() {
    let args = Args::parse();

    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
