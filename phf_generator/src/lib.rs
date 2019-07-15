#![doc(html_root_url="https://docs.rs/phf_generator/0.7")]
use phf_shared::{PhfHash, HashKey, Hashes};
use rand::{SeedableRng, Rng};
use rand::distributions::Standard;
use rand::rngs::SmallRng;

use std::mem;

const DEFAULT_LAMBDA: usize = 5;

const FIXED_SEED: u64 = 1234567890;

pub struct HashState {
    pub key: HashKey,
    pub disps: Vec<(u32, u32)>,
    pub map: Vec<usize>,
}

struct Bucket {
    idx: usize,
    keys: Vec<usize>,
}

struct GenCtxt {
    hashes: Vec<Hashes>,
    buckets: Vec<Bucket>,
    map: Vec<Option<usize>>,
    disps: Vec<(u32, u32)>,
    try_map: Vec<u64>,
    values_to_add: Vec<(usize, usize)>,
}

impl GenCtxt {
    fn reset(&mut self) {
        self.hashes.clear();

        for (i, bucket) in self.buckets.iter_mut().enumerate() {
            bucket.idx = i;
            bucket.keys.clear();
        }

        overwrite_all(&mut self.map, None);
        overwrite_all(&mut self.disps, (0, 0));
        overwrite_all(&mut self.try_map, 0);
        self.values_to_add.clear();
    }
}

fn overwrite_all<T: Copy>(data: &mut [T], val: T) {
    for place in data {
        *place = val;
    }
}

pub fn generate_hash<H: PhfHash>(entries: &[H]) -> HashState {
    let buckets_len = (entries.len() + DEFAULT_LAMBDA - 1) / DEFAULT_LAMBDA;

    // allocate all scratch space up front and reusze it
    let mut ctxt = GenCtxt {
        hashes: Vec::with_capacity(entries.len()),
        buckets: (0..buckets_len)
            .map(|i| {
                Bucket {
                    idx: i,
                    keys: vec![],
                }
            })
            .collect(),
        map: vec![None; entries.len()],
        disps: vec![(0, 0); buckets_len],
        try_map: vec![0u64; entries.len()],
        values_to_add: Vec::with_capacity(16),
    };

    SmallRng::seed_from_u64(FIXED_SEED)
        .sample_iter(Standard)
        .find_map(|key| try_generate_hash(&mut ctxt, entries, key))
        .expect("failed to solve PHF")
}

fn try_generate_hash<H: PhfHash>(ctxt: &mut GenCtxt, entries: &[H], key: HashKey) -> Option<HashState> {
    assert!(ctxt.hashes.is_empty(), "GenCtxt.reset() was not called");

    ctxt.hashes.extend(entries.iter().map(|entry| phf_shared::hash(entry, &key)));

    let GenCtxt {
        ref mut buckets,
        ref hashes,
        ref mut map,
        ref mut disps,
        ref mut try_map,
        ref mut values_to_add,
    } = *ctxt;

    let buckets_len = buckets.len();
    for (i, hash) in hashes.iter().enumerate() {
        buckets[(hash.g % (buckets_len as u32)) as usize].keys.push(i);
    }

    // Sort descending
    buckets.sort_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());

    // store whether an element from the bucket being placed is
    // located at a certain position, to allow for efficient overlap
    // checks. It works by storing the generation in each cell and
    // each new placement-attempt is a new generation, so you can tell
    // if this is legitimately full by checking that the generations
    // are equal. (A u64 is far too large to overflow in a reasonable
    // time for current hardware.)
    // let mut try_map = vec![0u64; table_len];
    let mut generation = 0u64;

    // the actual values corresponding to the markers above, as
    // (index, key) pairs, for adding to the main map once we've
    // chosen the right disps.
    // let mut values_to_add = vec![];

    let table_len = entries.len();

    'buckets: for bucket in &*buckets {
        for d1 in 0..(table_len as u32) {
            'disps: for d2 in 0..(table_len as u32) {
                values_to_add.clear();
                generation += 1;

                for &key in &bucket.keys {
                    let idx = (phf_shared::displace(hashes[key].f1, hashes[key].f2, d1, d2) %
                               (table_len as u32)) as usize;
                    if map[idx].is_some() || try_map[idx] == generation {
                        continue 'disps;
                    }
                    try_map[idx] = generation;
                    values_to_add.push((idx, key));
                }

                // We've picked a good set of disps
                disps[bucket.idx] = (d1, d2);
                for &(idx, key) in &*values_to_add {
                    map[idx] = Some(key);
                }
                continue 'buckets;
            }
        }

        // Unable to find displacements for a bucket
        ctxt.reset();
        return None;
    }

    Some(HashState {
        key,
        disps: mem::replace(disps, Vec::new()),
        map: map.drain(..).map(|i| i.unwrap()).collect(),
    })
}
