use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

use base_62::base62;

const ID_CHARS: [char; 62] = [
    'A', 'Z', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'Q', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L',
    'M', 'W', 'X', 'C', 'V', 'B', 'N', 'a', 'z', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', 'q', 's',
    'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm', 'w', 'x', 'c', 'v', 'b', 'n', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9',
];
const ID_SIZE: usize = 5;

trait IdBase {
    type Type;
}

pub type IdValue = u32;

#[derive(Debug)]
pub struct Id<T> {
    pub value: IdValue,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new(value: IdValue) -> Self {
        Id {
            value,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T: typename::TypeName> fmt::Display for Id<T> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.

        let x: [u8; 4] = unsafe { std::mem::transmute(self.value.to_le()) };
        write!(
            f,
            "{:?} {}",
            T::type_name(),
            base62::encode(&x) // format!("{:X}", self.value)
        )
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self::new(self.value)
    }
}
impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Hash for Id<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.value.hash(state);
    }
}

impl<T> Eq for Id<T> {}

impl<T> IdBase for Id<T> {
    type Type = T;
}

pub fn rand_id<T>() -> Id<T> {
    Id::new(rand::prelude::random())
}

pub fn rand_id_unsafe() -> String {
    let mut rng = thread_rng();
    let mut s = String::with_capacity(ID_SIZE);
    for _ in 0..ID_SIZE {
        s.push(*ID_CHARS.choose(&mut rng).unwrap());
    }
    s
}

pub fn pop_set<T: Clone + Eq + std::hash::Hash>(set: &mut HashSet<T>) -> T {
    let elt = set.iter().next().cloned().unwrap();
    set.take(&elt).unwrap()
}

pub fn time<F, K>(f: F) -> u128
where
    F: FnOnce() -> K,
{
    let start = std::time::Instant::now();
    f();
    start.elapsed().as_micros()
}