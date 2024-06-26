use clap::Parser;
use count_cat::ThreeFileArgs;
use noodles_bgzf as bgzf;
use noodles_bgzf::io::Seek;
use noodles_fasta as fasta;
use noodles_fasta::fai::Record as FaiRecord;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;

fn cat_count(
    fai_rec: &FaiRecord,
    bgzf_index: &bgzf::gzi::Index,
    input_path: &PathBuf,
) -> Result<usize, io::Error> {
    let mut buf: Vec<u8> = Vec::new();
    let mut bgzf_reader = bgzf::Reader::new(File::open(input_path)?);
    bgzf_reader.seek_with_index(bgzf_index, io::SeekFrom::Start(fai_rec.offset()))?;
    let mut fasta_reader = fasta::io::Reader::new(BufReader::new(bgzf_reader));
    fasta_reader.read_sequence(&mut buf)?;
    buf.iter_mut().for_each(|b| *b = b.to_ascii_uppercase());
    Ok(buf
        .as_slice()
        .windows(3)
        .filter(|&win| win == b"CAT")
        .count())
}

fn main() -> io::Result<()> {
    let opts = ThreeFileArgs::parse();
    let fai_file = File::open(&opts.fai_path)?;
    let bgzf_index = bgzf::gzi::read(&opts.gzi_path)?;
    let mut index = fasta::fai::Reader::new(BufReader::new(fai_file));
    let index_records = index.read_index()?;
    let input_path = opts.input_path;
    let cat_counted = index_records
        .par_iter()
        .map(|fai_rec| cat_count(&fai_rec, &bgzf_index, &input_path))
        .try_fold(
            || 0 as usize, // Initial accumulator for each thread
            |acc, res| match res {
                Ok(value) => Ok(acc + value),
                Err(e) => Err(e),
            },
        )
        .try_reduce(
            || 0, // Initial accumulator for combining results
            |acc1, acc2| Ok(acc1 + acc2),
        );
    match cat_counted {
        Ok(total) => println!("Number of CATs: {}", total),
        Err(_) => println!("Error counting CATs"),
    }
    Ok(())
}
