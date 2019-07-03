#![doc(html_root_url="https://docs.rs/phf_generator/0.7")]
extern crate phf_shared;
extern crate rand;

use phf_shared::PhfHash;
use rand::{SeedableRng, Rng};
use rand::rngs::SmallRng;

const DEFAULT_LAMBDA: usize = 5;

const FIXED_SEED: u64 = 0xAAAAAAAAAAAAAAAA;

#[derive(Debug)]
pub struct HashState {
    pub keys: [u64; 3],
    pub disps: Vec<(u32, u32)>,
    pub map: Vec<usize>,
}

pub fn generate_hash<H: PhfHash>(entries: &[H]) -> HashState {
    let mut rng = SmallRng::seed_from_u64(FIXED_SEED);
    loop {
        if let Some(s) = try_generate_hash(entries, &mut rng) {
            return s;
        } else {
            println!("Finding hash failed, retrying");
        }
    }
}

fn try_generate_hash<H: PhfHash>(entries: &[H], rng: &mut SmallRng) -> Option<HashState> {
    struct Bucket {
        idx: usize,
        keys: Vec<usize>,
    }

    struct Hashes {
        g: u32,
        f1: u32,
        f2: u32,
    }

    let key1 = rng.gen();
    let key2 = rng.gen();
    let key3 = rng.gen();

    let keys = [key1, key2, key3];

    let hashes: Vec<_> = entries.iter()
                                .map(|entry| {
                                    let [g, f1, f2] = phf_shared::hash(entry, keys);

                                    Hashes {
                                        g, f1, f2,
                                    }
                                })
                                .collect();

    // We want the number of buckets to be rounded up.
    let buckets_len = (entries.len() + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA;
    let mut buckets = (0..buckets_len)
                          .map(|i| {
                              Bucket {
                                  idx: i,
                                  keys: vec![],
                              }
                          })
                          .collect::<Vec<_>>();

    // Sort into buckets by value of hash
    for (i, hash) in hashes.iter().enumerate() {
        buckets[(hash.g % (buckets_len as u32)) as usize].keys.push(i);
    }

    // Sort descending
    buckets.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());

    let table_len = entries.len();
    let mut map = vec![None; table_len];
    let mut disps = vec![(0u32, 0u32); buckets_len];

    // store whether an element from the bucket being placed is
    // located at a certain position, to allow for efficient overlap
    // checks. It works by storing the generation in each cell and
    // each new placement-attempt is a new generation, so you can tell
    // if this is legitimately full by checking that the generations
    // are equal. (A u64 is far too large to overflow in a reasonable
    // time for current hardware.)
    let mut try_map = vec![0u64; table_len];
    let mut generation = 0u64;

    // the actual values corresponding to the markers above, as
    // (index, key) pairs, for adding to the main map once we've
    // chosen the right disps.
    let mut values_to_add = vec![];

    // For debugging - see how many d1 and d2s it takes.
    let mut track_attempts = vec![];
    let mut track_count = 0u64;

    'buckets: for bucket in &buckets {
        let mut attempts = 0u64;
        for d1 in 0..(table_len as u32) {
            'disps: for d2 in 0..(table_len as u32) {
                values_to_add.clear();
                generation += 1;

                for &key in &bucket.keys {
                    let idx = (phf_shared::displace(hashes[key].f1, hashes[key].f2, d1, d2) %
                               (table_len as u32)) as usize;
                    if map[idx].is_some() || try_map[idx] == generation {
                        attempts += 1;
                        continue 'disps;
                    }
                    try_map[idx] = generation;
                    values_to_add.push((idx, key));
                }

                // We've picked a good set of disps
                disps[bucket.idx] = (d1, d2);
                for &(idx, key) in &values_to_add {
                    map[idx] = Some(key);
                }
                //println!("Bucket {} took {} attempts", track_count, attempts);
                track_count += 1;
                track_attempts.push(attempts);
                continue 'buckets;
            }
        }

        // Unable to find displacements for a bucket
        return None;
    }

    let count = track_attempts.iter().count() as u64;
    let sum = track_attempts.iter().sum::<u64>();
    // println!("count: {}, sum: {}, average: {}", count, sum, sum / count);

    Some(HashState {
        keys,
        disps,
        map: map.into_iter().map(|i| i.unwrap()).collect(),
    })
}
