use std::collections::hash_map::Keys;
use std::fmt::{Debug};
use std::hash::Hash;
use core::convert::From;

use rand::RngExt;
use rand::rngs::ThreadRng;
use rapidhash::RapidHashMap;

use crate::game_logic::RapidIndexMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MultiHashSet<T: Eq + Hash> {
    elements: RapidIndexMap<T, u32>
}

impl<T: Eq + Hash + Copy> Extend<T> for MultiHashSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for el in iter {
            self.put(el);
        }
    }
}

impl<T: Eq + Hash + Copy> FromIterator<T> for MultiHashSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = MultiHashSet::new();
        set.extend(iter);
        set
    }
}

impl<K: Eq + Hash + Copy> FromIterator<(K, u32)> for MultiHashSet<K> {
    fn from_iter<I: IntoIterator<Item = (K, u32)>>(iter: I) -> Self {
        let mut set = MultiHashSet::new();
        for (k, v) in iter.into_iter() {
            set.set(k, v);
        }
        set
    }
}

impl<T: Eq + Hash + Copy, const N: usize> From<[T;N]> for MultiHashSet<T> {
    fn from(arr: [T;N]) -> Self {
        Self::from_iter(arr)
    }
}

impl<T: Eq + Hash + Copy> MultiHashSet<T> {
    pub fn new() -> MultiHashSet<T> {
        MultiHashSet { elements: RapidIndexMap::default() }
    }

    pub fn set(&mut self, k: T, v: u32) {
        self.elements.insert(k, v);
    }

    pub fn put(&mut self, el: T) {
        *self.elements.entry(el).or_default() += 1
    }

    pub fn take(&mut self, el: &T) {
        self.elements.entry(*el).and_modify(|x| *x = x.saturating_sub(1));
        if let Some(0) = self.elements.get(el) {
            self.elements.swap_remove(el);
        }
    }

    pub fn elements(&self) -> indexmap::map::Keys<'_, T, u32>{
        self.elements.keys()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.elements.keys()
    }

    pub fn get_random(&self, rng: &mut ThreadRng) -> &T {
        let idx = rng.random_range(0..self.elements.len());
        self.elements.get_index(idx).unwrap().0
    }
}
