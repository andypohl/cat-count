use clap::Parser;
use noodles_fasta as fasta;
use noodles_fasta::fai::Record as FaiRecord;

use rayon::prelude::*;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Seek;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct CliOpts {
    /// File path
    #[arg(required = true, value_parser = path_exists)]
    pub input_path: PathBuf,

    #[arg(required = true, value_parser = path_exists)]
    pub fai_path: PathBuf,
}

/// Custom validator to check if a path exists
fn path_exists(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!(
            "The specified path '{}' does not exist",
            path.display()
        ))
    }
}

fn g_count(fai_rec: &FaiRecord, input_path: &PathBuf) -> usize {
    let mut buf: Vec<u8> = Vec::new();
    let mut input_file = File::open(input_path).unwrap();
    input_file
        .seek(io::SeekFrom::Start(fai_rec.offset()))
        .unwrap();
    let mut fasta_reader = fasta::io::Reader::new(BufReader::new(input_file));
    fasta_reader.read_sequence(&mut buf).unwrap();
    buf.iter().filter(|&b| matches!(*b, b'G' | b'g')).count()
}

fn main() -> io::Result<()> {
    let opts = CliOpts::parse();
    let fai_file = File::open(&opts.fai_path)?;
    let mut index = fasta::fai::Reader::new(BufReader::new(fai_file));
    let index_records = index.read_index().unwrap();
    let input_path = opts.input_path;
    let g_counted: usize = index_records
        .par_iter()
        .map(|fai_rec| g_count(&fai_rec, &input_path))
        .sum();
    println!("Number of Gs: {}", g_counted);

    Ok(())
}
