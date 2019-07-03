#![doc(html_root_url="https://docs.rs/phf_shared/0.7")]
#![cfg_attr(feature = "core", no_std)]

#[cfg(not(feature = "core"))]
extern crate std as core;

extern crate siphasher;
extern crate fnv;

#[cfg(feature = "unicase")]
extern crate unicase;

use core::fmt;
use core::hash::{Hasher, Hash};
use siphasher::sip::SipHasher13;
use fnv::FnvHasher;

#[inline]
pub fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2)
}

/// `key` is from `phf_generator::HashState::key`.
#[inline]
pub fn hash<T: ?Sized + PhfHash>(x: &T, keys: [u64; 3]) -> [u32; 3] {
    let mut hashes = [0; 3];

    for (i, (&key, hash_out)) in keys.iter().zip(&mut hashes).enumerate() {
        // let mut hasher = FnvHasher::with_key(key);
        let mut hasher = SipHasher13::new_with_keys(i as u64, key);
        x.phf_hash(&mut hasher);
        *hash_out = (hasher.finish() >> 32) as u32;
    }

    hashes
}

/// Return an index into `phf_generator::HashState::map`.
///
/// * `hash` is from `hash()` in this crate.
/// * `disps` is from `phf_generator::HashState::disps`.
/// * `len` is the length of `phf_generator::HashState::map`.
#[inline]
pub fn get_index(hash: [u32; 3], disps: &[(u32, u32)], len: usize) -> u32 {
    let [g, f1, f2] = hash;
    let (d1, d2) = disps[(g % (disps.len() as u32)) as usize];
    displace(f1, f2, d1, d2) % (len as u32)
}

/// A trait implemented by types which can be used in PHF data structures.
///
/// This differs from the standard library's `Hash` trait in that `PhfHash`'s
/// results must be architecture independent so that hashes will be consistent
/// between the host and target when cross compiling.
pub trait PhfHash {
    /// Feeds the value into the state given, updating the hasher as necessary.
    fn phf_hash<H: Hasher>(&self, state: &mut H);

    /// Feeds a slice of this type into the state provided.
    fn phf_hash_slice<H: Hasher>(data: &[Self], state: &mut H)
        where Self: Sized
    {
        for piece in data {
            piece.phf_hash(state);
        }
    }
}

/// Trait for printing types with `const` constructors, used by `phf_codegen` and `phf_macros`.
pub trait FmtConst {
    /// Print a `const` expression representing this value.
    fn fmt_const(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

/// Create an impl of `FmtConst` delegating to `fmt::Debug` for types that can deal with it.
///
/// Ideally with specialization this could be just one default impl and then specialized where
/// it doesn't apply.
macro_rules! delegate_debug(
    ($ty:ty) => {
        impl FmtConst for $ty {
            fn fmt_const(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    }
);

delegate_debug!(str);
delegate_debug!([u8]);
delegate_debug!(char);
delegate_debug!(u8);
delegate_debug!(i8);
delegate_debug!(u16);
delegate_debug!(i16);
delegate_debug!(u32);
delegate_debug!(i32);
delegate_debug!(u64);
delegate_debug!(i64);
delegate_debug!(bool);

#[cfg(not(feature = "core"))]
impl PhfHash for String {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (**self).phf_hash(state)
    }
}

#[cfg(not(feature = "core"))]
impl PhfHash for Vec<u8> {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (**self).phf_hash(state)
    }
}

impl<'a, T: 'a + PhfHash + ?Sized> PhfHash for &'a T {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self).phf_hash(state)
    }
}

impl<'a, T: 'a + FmtConst + ?Sized> FmtConst for &'a T {
    fn fmt_const(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self).fmt_const(f)
    }
}

impl PhfHash for str {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().phf_hash(state)
    }
}

impl PhfHash for [u8] {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        state.write(self);
    }
}

#[cfg(feature = "unicase")]
impl<S> PhfHash for unicase::UniCase<S>
where unicase::UniCase<S>: Hash {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

#[cfg(feature = "unicase")]
impl<S> FmtConst for unicase::UniCase<S> where S: AsRef<str> {
    fn fmt_const(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_ascii() {
            f.write_str("UniCase::ascii(")?;
        } else {
            f.write_str("UniCase::unicode(")?;
        }

        self.as_ref().fmt_const(f)?;
        f.write_str(")")
    }
}

macro_rules! sip_impl(
    (le $t:ty) => (
        impl PhfHash for $t {
            #[inline]
            fn phf_hash<H: Hasher>(&self, state: &mut H) {
                self.to_le().hash(state);
            }
        }
    );
    ($t:ty) => (
        impl PhfHash for $t {
            #[inline]
            fn phf_hash<H: Hasher>(&self, state: &mut H) {
                self.hash(state);
            }
        }
    )
);

sip_impl!(u8);
sip_impl!(i8);
sip_impl!(le u16);
sip_impl!(le i16);
sip_impl!(le u32);
sip_impl!(le i32);
sip_impl!(le u64);
sip_impl!(le i64);
sip_impl!(bool);

impl PhfHash for char {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self as u32).phf_hash(state)
    }
}

macro_rules! array_impl(
    ($t:ty, $n:expr) => (
        impl PhfHash for [$t; $n] {
            #[inline]
            fn phf_hash<H: Hasher>(&self, state: &mut H) {
                state.write(self);
            }
        }

        impl FmtConst for [$t; $n] {
            fn fmt_const(&self, f: &mut fmt::Formatter) -> fmt::Result {
                // delegate to the slice impl to minimize duplicated code
                self[..].fmt_const(f)
            }
        }
    )
);

array_impl!(u8, 1);
array_impl!(u8, 2);
array_impl!(u8, 3);
array_impl!(u8, 4);
array_impl!(u8, 5);
array_impl!(u8, 6);
array_impl!(u8, 7);
array_impl!(u8, 8);
array_impl!(u8, 9);
array_impl!(u8, 10);
array_impl!(u8, 11);
array_impl!(u8, 12);
array_impl!(u8, 13);
array_impl!(u8, 14);
array_impl!(u8, 15);
array_impl!(u8, 16);
array_impl!(u8, 17);
array_impl!(u8, 18);
array_impl!(u8, 19);
array_impl!(u8, 20);
array_impl!(u8, 21);
array_impl!(u8, 22);
array_impl!(u8, 23);
array_impl!(u8, 24);
array_impl!(u8, 25);
array_impl!(u8, 26);
array_impl!(u8, 27);
array_impl!(u8, 28);
array_impl!(u8, 29);
array_impl!(u8, 30);
array_impl!(u8, 31);
array_impl!(u8, 32);
