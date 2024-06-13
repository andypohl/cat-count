use clap::Parser;
use noodles_core::Region;
use noodles_fasta as fasta;
use std::fs::File;
use std::io;
use std::io::BufReader;
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

fn get_chrom_list(index_records: &Vec<fasta::fai::Record>) -> Vec<Vec<u8>> {
    let mut chrom_list = Vec::new();
    for record in index_records {
        chrom_list.push(record.name().to_owned());
    }
    chrom_list
}

fn main() -> io::Result<()> {
    let opts = CliOpts::parse();
    let fai_file = File::open(&opts.fai_path)?;
    let input_file = File::open(&opts.input_path)?;
    let mut index = fasta::fai::Reader::new(BufReader::new(fai_file));
    let index_records = index.read_index().unwrap();
    let chrom_list = get_chrom_list(&index_records);
    let mut fasta_reader = fasta::io::IndexedReader::new(BufReader::new(input_file), index_records);
    let g_count = chrom_list
        .iter()
        .map(|chrom_name| {
            let chrom = fasta_reader
                .query(&Region::new(chrom_name.to_owned(), ..))
                .unwrap();
            chrom
                .sequence()
                .as_ref()
                .iter()
                .filter(|&b| matches!(*b, b'G' | b'g'))
                .count()
        })
        .sum::<usize>();
    println!("Number of Gs: {}", g_count);

    Ok(())
}
