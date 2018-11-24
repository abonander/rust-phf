use phf_generator::try_generate_hash;
use rand::{SeedableRng, XorShiftRng};
use std::io::Write;

include!(concat!(env!("OUT_DIR"), "/passwords.rs"));

const FIXED_SEED: [u8; 16] = *b"\xec\x58\xdf\xa7\x46\x41\xaf\x52\xad\x0d\x16\xe7\x7d\x57\x66\x23";

fn main() {
    println!("Starting hashing");
    let mut rng = XorShiftRng::from_seed(FIXED_SEED);
    let mut attempt = 1;
    let h = loop {
        println!("Attempt {}", attempt);
        if let Some(h) = try_generate_hash(&PASSWORDS, &mut rng) {
            break h;
        }
        attempt += 1;
    };
    let mut out = std::fs::File::create("/dev/null").unwrap();
    write!(out, "{:?}", h).unwrap();
}
