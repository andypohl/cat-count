use clap::Parser;
use noodles_core::{Position, Region};
use noodles_fasta as fasta;
use std::fs::File;
use std::io;
use std::io::BufReader;
//use std::io::Seek;
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

fn main() -> io::Result<()> {
    let opts = CliOpts::parse();
    let fai_file = File::open(&opts.fai_path)?;
    let mut index = fasta::fai::Reader::new(BufReader::new(&fai_file));
    let index_records = index.read_index().unwrap();
    let input_path = File::open(&opts.input_path)?;
    //input_path.seek(io::SeekFrom::Start(251492284))?;
    let mut fasta_reader = fasta::io::Reader::new(BufReader::new(input_path));
    let start = Position::try_from(1).unwrap();
    let end = Position::try_from(242696752).unwrap();
    let chr_2 = fasta_reader
        .query(&index_records, &Region::new("chr2", start..=end))
        .unwrap();
    /*     let mut bgzf_reader = bgzf::Reader::new(input_path);
    let mut fasta_reader = fasta::io::Reader::new(&mut bgzf_reader);
    */
    // 48914580

    let mut g_count = 0;
    for base in chr_2.sequence().as_ref().iter() {
        if matches!(*base, b'G' | b'g') {
            g_count += 1;
        }
    }
    println!("Number of Gs: {}", g_count);

    //for rec in records {
    //    match rec {
    //        Ok(record) => {
    //            let sequence = record.sequence().as_ref();
    //            for base in sequence {
    //                if *base == b'G' || *base == b'g' {
    //                    g_count += 1;
    //                }
    //            }
    //        }
    //        Err(e) => eprintln!("Error: {}", e),
    //    }
    //}

    Ok(())
}
