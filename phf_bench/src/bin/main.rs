use std::time::Instant;
use std::io::prelude::*;
use std::fs::File;

fn main() {
    let pws = phf_bench::get_pws(500_000);
    let start = Instant::now();
    let pws_hash = phf_generator::generate_hash(&pws);
    println!("Duration: {:?}", Instant::now() - start);
    let mut dummy = File::create("/dev/null").expect("opening /dev/null");
    write!(dummy, "{:?}", pws_hash).expect("writing to /dev/null");
}
