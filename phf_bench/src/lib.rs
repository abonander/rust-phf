#![feature(test, extern_crate_item_prelude)]
extern crate test;

use xz2::read::XzDecoder;
use std::fs::File;
use std::env;
use std::io::{self, prelude::*};
use std::path::Path;


pub fn get_pws(count: usize) -> Vec<String> {
    let input_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap_or(".".into()))
        .join("data/rockyou.txt.xz");
    let input = io::BufReader::new(XzDecoder::new(File::open(input_path).unwrap()));

    input.split(b'\n')
        .map(|line| line.unwrap())
        .filter_map(|word| String::from_utf8(word).ok())
        .take(count)
        .collect()

}

#[cfg(test)]
mod tests {
    use test::Bencher;
    use phf_generator::generate_hash;
    use super::get_pws;

    const PASSWORD_COUNT: usize = 100_000;

    #[bench]
    fn bench_100_000(b: &mut Bencher) {
        let pws = get_pws(PASSWORD_COUNT);
        b.iter(|| generate_hash(&pws))
    }
}
