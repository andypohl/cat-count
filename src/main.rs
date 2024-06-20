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

struct Triplets<I, T> {
    inner: I,
    buffer: [T; 3],
    length: usize,
}

impl<I, T, E> Triplets<I, T>
where
    I: Iterator<Item = Result<T, E>>,
    T: Default + Copy,
{
    fn new(inner: I) -> Self {
        Self {
            inner,
            buffer: [T::default(); 3],
            length: 0,
        }
    }
}

impl<I, T, E> Iterator for Triplets<I, T>
where
    I: Iterator<Item = Result<T, E>>,
    T: Clone + Copy + Default,
{
    type Item = [T; 3];

    fn next(&mut self) -> Option<Self::Item> {
        while self.length < 2 {
            match self.inner.next() {
                Some(Ok(t)) => {
                    self.buffer[self.length + 1] = t;
                    self.length += 1;
                }
                _ => return None,
            }
        }
        match self.inner.next() {
            Some(Ok(t)) => {
                self.buffer.rotate_left(1);
                self.buffer[2] = t;
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

fn g_count(fai_rec: &FaiRecord, bgzf_index: &bgzf::gzi::Index, input_path: &PathBuf) -> usize {
    let mut bgzf_reader = bgzf::Reader::new(File::open(input_path).unwrap());
    bgzf_reader
        .seek_with_index(bgzf_index, io::SeekFrom::Start(fai_rec.offset()))
        .unwrap();
    let mut fasta_reader = fasta::io::Reader::new(BufReader::new(bgzf_reader));
    Triplets::new(fasta_reader.sequence_reader().bytes())
        .filter(|trip| trip.to_ascii_uppercase() == b"CAT")
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
        .map(|fai_rec| g_count(&fai_rec, &bgzf_index, &input_path))
        .sum();
    println!("Number of CATs: {}", cats_counted);

    Ok(())
}
