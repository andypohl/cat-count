// cargo add crossbeam-channel@0.5.13
// cargo add noodles@0.77.0 --features bgzf,fasta
// cargo add noodles-bgzf@0.31.0 --features libdeflate

use std::{
    env,
    fs::File,
    io::{self, SeekFrom},
    num::NonZeroUsize,
    path::Path,
    thread,
};

use noodles::{
    bgzf::{self, gzi, io::Seek},
    fasta::{self, fai},
};

const PATTERN: &[u8; 3] = b"CAT";

fn main() -> io::Result<()> {
    let src = env::args().nth(1).expect("missing src");

    let fai_src = format!("{src}.fai");
    let fasta_index = fai::read(fai_src)?;

    let gzi_src = format!("{src}.gzi");
    let gz_index = gzi::read(gzi_src)?;

    let worker_count = thread::available_parallelism().unwrap_or(NonZeroUsize::MIN);

    let n = count(worker_count, src, &fasta_index, &gz_index)?;
    println!("{n}");

    Ok(())
}

fn count<P>(
    worker_count: NonZeroUsize,
    src: P,
    fasta_index: &fai::Index,
    gz_index: &gzi::Index,
) -> io::Result<usize>
where
    P: AsRef<Path>,
{
    let src = src.as_ref();

    let (count_tx, count_rx) = crossbeam_channel::bounded(worker_count.get());
    let (result_tx, result_rx) = crossbeam_channel::bounded(worker_count.get());

    thread::scope(move |scope| {
        scope.spawn(move || {
            for record in fasta_index {
                count_tx.send(record).unwrap();
            }
        });

        for _ in 0..worker_count.get() {
            let count_rx = count_rx.clone();
            let result_tx = result_tx.clone();

            scope.spawn(move || {
                while let Ok(record) = count_rx.recv() {
                    let mut decoder = File::open(src).map(bgzf::Reader::new)?;
                    decoder.seek_with_index(gz_index, SeekFrom::Start(record.offset()))?;

                    let mut reader = fasta::io::Reader::new(decoder);

                    let len = usize::try_from(record.length())
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    let mut buf = Vec::with_capacity(len);
                    reader.read_sequence(&mut buf)?;

                    buf.make_ascii_uppercase();

                    let n = buf
                        .windows(PATTERN.len())
                        .filter(|buf| buf == PATTERN)
                        .count();

                    result_tx.send(n).unwrap();
                }

                Ok::<_, io::Error>(())
            });
        }

        drop(result_tx);

        let mut sum = 0;

        while let Ok(n) = result_rx.recv() {
            sum += n;
        }

        Ok(sum)
    })
}
