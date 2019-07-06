use bytesize::ByteSize;

use std::time::Instant;
use std::io::prelude::*;
use std::io;

fn main() {
    let pws = phf_bench::get_pws(1_000_000);
    let start = Instant::now();
    let pws_hash = phf_generator::generate_hash(&pws);
    println!("Duration: {:?}", Instant::now() - start);
    let output = format!("{:?}", pws_hash);
    println!("Encoded Size: ~{}", ByteSize::b(output.len() as u64));
}
