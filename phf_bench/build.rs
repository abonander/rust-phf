use xz2::read::XzDecoder;
use std::fs::File;
use std::path::Path;
use std::io::{self, prelude::*};

const PASSWORD_COUNT: usize = 200_000;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("passwords.rs");
    let mut out = File::create(&dest_path).unwrap();

    let input_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/rockyou.txt.xz");
    let input = io::BufReader::new(XzDecoder::new(File::open(input_path).unwrap()));

    let words: Vec<_> = input.split(b'\n')
        .map(|line| line.unwrap())
        .filter_map(|word| String::from_utf8(word).ok())
        .take(PASSWORD_COUNT)
        .collect();

    write!(out, "const PASSWORDS: [&'static str; {}] = [", words.len()).unwrap();
    for word in words {
        write!(out, "{:?}, ", word).unwrap();
    }
    write!(out, "];\n").unwrap();
}
