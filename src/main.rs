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

    /// Input files
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn run(args: Args) -> Result<(), AzadiError> {
    // Create the SafeFileWriter
    let safe_writer = SafeFileWriter::new(args.gen, args.priv_dir);
    let mut clipper = Clip::new(safe_writer);

    // Read all input files
    clipper.read_files(&args.files)?;

    // Write files
    clipper.write_files()?;

    // Handle specific chunks if requested
    if let Some(chunks) = args.chunks {
        let chunks: Vec<&str> = chunks.split(',').collect();
        if let Some(output_path) = args.output {
            let mut file = File::create(output_path)?;
            for chunk in chunks {
                clipper.get_chunk(chunk, &mut file)?;
                writeln!(file)?;
            }
        } else {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            for chunk in chunks {
                clipper.get_chunk(chunk, &mut handle)?;
                writeln!(handle)?;
            }
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
