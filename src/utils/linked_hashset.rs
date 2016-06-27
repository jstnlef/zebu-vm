extern crate linked_hash_map;

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};
use std::borrow::Borrow;

use self::linked_hash_map::Keys;
pub use self::linked_hash_map::LinkedHashMap;

pub struct LinkedHashSet<K, S = RandomState>(LinkedHashMap<K, (), S>);

impl<K: Hash + Eq> LinkedHashSet<K> {
    pub fn new() -> Self {
        LinkedHashSet(LinkedHashMap::new())
    }
}

impl<K: Hash + Eq, S: BuildHasher> LinkedHashSet<K, S> {
    pub fn insert(&mut self, k: K) -> Option<()> {
        self.0.insert(k, ())
    }

    pub fn contains<Q: ?Sized>(&self, k: &Q) -> bool
        where K: Borrow<Q>,
              Q: Eq + Hash
    {
        self.0.contains_key(k)
    }

    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<()>
        where K: Borrow<Q>,
              Q: Eq + Hash
    {
        self.0.remove(k)
    }

    pub fn iter(&self) -> Keys<K, ()> {
        self.0.keys()
    }
    
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    
    pub fn pop_front(&mut self) -> Option<K> {
        match self.0.pop_front() {
            Some((k, _)) => Some(k),
            None => None
        }
    }
    
    pub fn add_all(&mut self, mut other: Self) {
        while !other.is_empty() {
            let entry = other.pop_front().unwrap();
            self.insert(entry);
        }
    }
}

impl<K: Hash + Eq + Clone> Clone for LinkedHashSet<K> {
    fn clone(&self) -> Self {
        LinkedHashSet(self.0.clone())
    }
}

use std::fmt;
impl<A: fmt::Debug + Hash + Eq, S: BuildHasher> fmt::Debug for LinkedHashSet<A, S> {
    /// Returns a string that lists the key-value pairs in insertion order.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}