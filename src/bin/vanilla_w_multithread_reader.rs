use clap::Parser;
use core::num::NonZeroUsize;
use noodles_bgzf as bgzf;
use noodles_fasta::io::Reader;
use std::fs::File;
use std::path::PathBuf;
use std::{io, thread};

#[derive(Parser, Debug)]
struct CliOpts {
    /// File path
    #[arg(required = true)]
    pub input_path: PathBuf,
}

fn main() -> io::Result<()> {
    let opts = CliOpts::parse();
    // Open the BGZF-compressed FASTA file
    let file = File::open(opts.input_path)?;
    let worker_count = thread::available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap());
    // Create a BGZF multithreaded reader
    let mut bgzf_reader = bgzf::MultithreadedReader::with_worker_count(worker_count, file); // 4 is the number of threads

    // Wrap the BGZF reader with the FASTA buffer reader
    let mut fasta_reader = Reader::new(&mut bgzf_reader);
    let records = fasta_reader.records();
    let mut cat_count = 0;
    // Read and print the sequences
    for rec in records {
        match rec {
            Ok(record) => {
                cat_count += record
                    .sequence()
                    .as_ref()
                    .to_ascii_uppercase()
                    .windows(3)
                    .filter(|&win| win == b"CAT")
                    .count();
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    println!("Number of CATs: {}", cat_count);

    Ok(())
}
