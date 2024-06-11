use clap::Parser;
use core::num::NonZeroUsize;
use noodles_bgzf as bgzf;
use noodles_fasta::io::Reader;
use std::fs::File;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct CliOpts {
    /// File path
    #[arg(required = true)]
    pub input_path: PathBuf,

    /// Number of workers
    #[arg(short, default_value = "1")]
    num_workers: NonZeroUsize,
}

fn main() -> io::Result<()> {
    let opts = CliOpts::parse();
    // Open the BGZF-compressed FASTA file
    let file = File::open(opts.input_path)?;

    // Create a BGZF multithreaded reader
    let mut bgzf_reader = bgzf::MultithreadedReader::with_worker_count(opts.num_workers, file); // 4 is the number of threads

    // Wrap the BGZF reader with the FASTA buffer reader
    let mut fasta_reader = Reader::new(&mut bgzf_reader);
    let records = fasta_reader.records();
    let mut g_count = 0;
    // Read and print the sequences
    for rec in records {
        match rec {
            Ok(record) => {
                let sequence = record.sequence().as_ref();
                for base in sequence {
                    if *base == b'G' || *base == b'g' {
                        g_count += 1;
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    println!("Number of Gs: {}", g_count);

    Ok(())
}
