use clap::Parser;
use noodles_bgzf as bgzf;
use noodles_bgzf::io::Seek;
use noodles_fasta as fasta;
use noodles_fasta::fai::Record as FaiRecord;
use rayon::prelude::*;
//use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::PathBuf;

trait IteratorExt: Iterator<Item = Result<u8, std::io::Error>>
where
    Self: Sized,
{
    fn triplets(self) -> Triplets<Self> {
        Triplets::new(self)
    }
}

impl<I> IteratorExt for I where I: Iterator<Item = Result<u8, std::io::Error>> + Sized {}

struct Triplets<I>
where
    I: Iterator<Item = Result<u8, std::io::Error>>,
{
    inner: I,
    buffer: [u8; 3],
}

impl<I> Triplets<I>
where
    I: Iterator<Item = Result<u8, std::io::Error>>,
{
    fn new(mut inner: I) -> Self {
        let mut buffer = [0; 3];
        for (i, val) in inner.by_ref().take(2).enumerate() {
            buffer[i] = val.unwrap().to_ascii_uppercase();
        }
        Self { inner, buffer }
    }
}

impl<I> Iterator for Triplets<I>
where
    I: Iterator<Item = Result<u8, std::io::Error>>,
{
    type Item = [u8; 3];

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(Ok(val)) => {
                self.buffer.rotate_left(1);
                self.buffer[2] = val.to_ascii_uppercase();
                Some(self.buffer)
            }
            _ => None,
        }
    }
}

#[derive(Parser, Debug)]
struct CliOpts {
    /// File path
    #[arg(required = true, value_parser = path_exists)]
    pub input_path: PathBuf,

    #[arg(required = true, value_parser = path_exists)]
    pub fai_path: PathBuf,

    #[arg(required = true, value_parser = path_exists)]
    pub gzi_path: PathBuf,
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

fn cat_count(fai_rec: &FaiRecord, bgzf_index: &bgzf::gzi::Index, input_path: &PathBuf) -> usize {
    let mut bgzf_reader = bgzf::Reader::new(File::open(input_path).unwrap());
    bgzf_reader
        .seek_with_index(bgzf_index, io::SeekFrom::Start(fai_rec.offset()))
        .unwrap();
    let mut fasta_reader = fasta::io::Reader::new(BufReader::new(bgzf_reader));
    fasta_reader
        .sequence_reader()
        .bytes()
        .triplets()
        .filter(|trip| trip == b"CAT")
        .count()
}

fn main() -> io::Result<()> {
    let opts = CliOpts::parse();
    let fai_file = File::open(&opts.fai_path)?;
    let bgzf_index = bgzf::gzi::read(&opts.gzi_path).unwrap();
    let mut index = fasta::fai::Reader::new(BufReader::new(fai_file));
    let index_records = index.read_index().unwrap();
    let input_path = opts.input_path;
    let cats_counted: usize = index_records
        .par_iter()
        .map(|fai_rec| cat_count(&fai_rec, &bgzf_index, &input_path))
        .sum();
    println!("Number of CATs: {}", cats_counted);
    Ok(())
}
