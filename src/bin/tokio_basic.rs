use async_trait::async_trait;
use clap::Parser;
use count_cat::OneFileArg;
use noodles_bgzf as bgzf;
use noodles_fasta::{r#async::io::Reader, AsyncReader};
use std::io;
use tokio::fs::File;
use tokio::io::AsyncBufRead;

/// Make an extension trait for the noodles async FASTA reader
#[async_trait]
trait ReaderExt {
    async fn just_read_sequence(&mut self) -> io::Result<Option<Vec<u8>>>;
}

#[async_trait]
impl<R> ReaderExt for Reader<R>
where
    R: AsyncBufRead + Unpin + Send,
{
    /// This method essentially just combines the read_definition and
    /// read_sequence methods
    async fn just_read_sequence(&mut self) -> io::Result<Option<Vec<u8>>> {
        let mut sequence = Vec::new();
        if (self.read_definition(&mut String::new()).await? == 0)
            || (self.read_sequence(&mut sequence).await? == 0)
        {
            return Ok(None);
        }
        sequence.iter_mut().for_each(|b| {
            *b = b.to_ascii_uppercase();
        });
        Ok(Some(sequence))
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let opts = OneFileArg::parse();
    let file = File::open(opts.input_path).await?;
    let mut bgzf_reader = bgzf::AsyncReader::new(file);
    let mut fasta_reader = AsyncReader::new(&mut bgzf_reader);
    let mut cat_count = 0;
    while let Some(seq_vec) = fasta_reader.just_read_sequence().await? {
        cat_count += seq_vec
            .as_slice()
            .windows(3)
            .filter(|&window| window == b"CAT")
            .count();
    }
    println!("Number of CATs: {}", cat_count);
    Ok(())
}
