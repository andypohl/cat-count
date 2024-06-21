# cat-count
Counts "CAT" in BGZF compressed/indexed FASTA files with several Rust readers.

## Running

Below shows a crude comparison running the three binaries on an M1 Mac Mini from 2020 with 16GB RAM.

```bash
cargo build -r --bin rayon_basic
gtime cargo run -r --bin rayon_basic -- chm13v2.0.fa.gz chm13v2.0.fa.gz.fai chm13v2.0.fa.gz.gzi
## Number of CATs: 60184687
## 7.30user 2.24system 0:02.55elapsed 374%CPU (0avgtext+0avgdata 1378128maxresident)k 0inputs+0outputs (2major+442501minor)pagefaults 0swaps
gtime cargo run -r --bin rayon_basic -- chm13v2.0.fa.gz chm13v2.0.fa.gz.fai chm13v2.0.fa.gz.gzi
cargo build -r --bin tokio_basic 
gtime cargo run -r --bin tokio_basic -- chm13v2.0.fa.gz                                                          
## Number of CATs: 60184687
## 6.57user 1.79system 0:03.71elapsed 224%CPU (0avgtext+0avgdata 330192maxresident)k 0inputs+0outputs (2major+428811minor)pagefaults 0swaps
cargo build -r --bin rayon_stream
gtime cargo run -r --bin rayon_stream -- chm13v2.0.fa.gz chm13v2.0.fa.gz.fai chm13v2.0.fa.gz.gzi
## Number of CATs: 60184687
## 66.08user 0.41system 0:11.38elapsed 584%CPU (0avgtext+0avgdata 5456maxresident)k 0inputs+0outputs (2major+4120minor)pagefaults 0swaps
```
