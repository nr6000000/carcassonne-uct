use std::collections::hash_map::Keys;
use std::collections::{HashMap};
use std::fmt::{Debug};
use std::hash::Hash;
use core::convert::From;

#[derive(Debug, PartialEq, Eq)]
pub struct MultiHashSet<T: Eq + Hash> {
    elements: HashMap<T, u32>
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

impl<T: Eq + Hash + Copy, const N: usize> From<[T;N]> for MultiHashSet<T> {
    fn from(arr: [T;N]) -> Self {
        Self::from_iter(arr)
    }
}

impl<T: Eq + Hash + Copy> MultiHashSet<T> {
    pub fn new() -> MultiHashSet<T> {
        MultiHashSet { elements: HashMap::new() }
    }

    pub fn put(&mut self, el: T) {
        *self.elements.entry(el).or_default() += 1
    }

    pub fn take(&mut self, el: T) {
        self.elements.entry(el).and_modify(|x| *x = x.saturating_sub(1));
        if let Some(0) = self.elements.get(&el) {
            self.elements.remove(&el);
        }
    }

    pub fn elements(&self) -> Keys<'_, T, u32> {
        self.elements.keys()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }
}
