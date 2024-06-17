use clap::Parser;
use noodles_core::Region;
use noodles_fasta as fasta;
use noodles_fasta::fai::Record as FaiRecord;

use rayon::prelude::*;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

trait CustomClone {
    fn custom_clone(&self) -> Self;
}

impl CustomClone for FaiRecord {
    fn custom_clone(&self) -> Self {
        // Clone each field individually
        FaiRecord::new(
            self.name(),
            self.length(),
            self.offset(),
            self.line_bases(),
            self.line_width(), // Clone other fields as necessary
        )
    }
}
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

fn get_chrom_list(index_records: &Vec<fasta::fai::Record>) -> Vec<Vec<u8>> {
    let mut chrom_list = Vec::new();
    for record in index_records {
        chrom_list.push(record.name().to_owned());
    }
    chrom_list
}

fn g_count(chrom: Vec<u8>, input_path: Arc<PathBuf>, index_records: Vec<FaiRecord>) -> usize {
    let input_file = File::open(input_path.as_ref()).unwrap();
    let mut fasta_reader = fasta::io::IndexedReader::new(BufReader::new(input_file), index_records);
    let chrom = fasta_reader
        .query(&Region::new(chrom.to_owned(), ..))
        .unwrap();
    chrom
        .sequence()
        .as_ref()
        .iter()
        .filter(|&b| matches!(*b, b'G' | b'g'))
        .count()
}

fn main() -> io::Result<()> {
    let opts = CliOpts::parse();
    let fai_file = File::open(&opts.fai_path)?;
    let mut index = fasta::fai::Reader::new(BufReader::new(fai_file));
    let index_records = index.read_index().unwrap();
    let chrom_list = get_chrom_list(&index_records);
    let input_path = Arc::new(opts.input_path);
    let g_counted: usize = chrom_list
        .par_iter()
        .map(|chrom| {
            g_count(
                chrom.to_owned(),
                input_path.clone(),
                index_records.iter().map(|r| r.custom_clone()).collect(),
            )
        })
        .sum();
    println!("Number of Gs: {}", g_counted);

    Ok(())
}
