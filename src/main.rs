use clap::Parser;
use noodles_bgzf as bgzf;
use noodles_fasta as fasta;
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
    let input_path = File::open(opts.input_path)?;
    let mut bgzf_reader = bgzf::Reader::new(input_path);
    let mut fasta_reader = fasta::io::Reader::new(&mut bgzf_reader);
    let records = fasta_reader.records();
    let mut g_count = 0;
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
