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
use std::vec::Vec;

struct Triplets<I, T> {
    inner: I,
    buffer: Vec<T>,
}

impl<I, T, E> Triplets<I, T>
where
    I: Iterator<Item = Result<T, E>>,
{
    fn new(inner: I) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
        }
    }
}

impl<I, T, E> Iterator for Triplets<I, T>
where
    I: Iterator<Item = Result<T, E>>,
    T: Clone,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.buffer.len() < 3 {
            if let Some(item) = self.inner.next() {
                match item {
                    Ok(thing) => {
                        self.buffer.push(thing);
                    }
                    Err(_) => return None,
                }
            } else {
                break;
            }
        }
        if self.buffer.len() == 3 {
            let val = self.buffer[..3].to_vec();
            self.buffer.remove(0);
            Some(val)
        } else {
            None
        }
    }
}

// Remove the conflicting implementation of Iterator for &mut Triplets<T>

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
