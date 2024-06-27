use clap::Parser;
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
}

fn main() -> io::Result<()> {
    let opts = CliOpts::parse();
    // Open the BGZF-compressed FASTA file
    let file = File::open(opts.input_path)?;
    // Create a BGZF single-threaded reader
    let mut bgzf_reader = bgzf::Reader::new(file);

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
