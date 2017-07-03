// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};
use std::borrow::Borrow;

use linked_hashmap::LinkedHashMap;
use linked_hashmap::Keys;

/// A LinkedHashSet based on LinkedHashMap implementation
pub struct LinkedHashSet<K, S = RandomState>(LinkedHashMap<K, (), S>);

impl<K: Hash + Eq> LinkedHashSet<K> {
    pub fn new() -> Self {
        LinkedHashSet(LinkedHashMap::new())
    }

    /// consumes a vector to a LinkedHashSet (removes duplicated elements)
    pub fn from_vec(from: Vec<K>) -> Self {
        let mut ret = LinkedHashSet::new();
        
        for ele in from {
            ret.insert(ele);
        }
        
        ret
    }

    /// consumes the LinkedHashSet to a vector
    pub fn to_vec(mut self) -> Vec<K> {
        let mut ret = vec![];

        while !self.is_empty() {
            ret.push(self.pop_front().unwrap());
        }

        ret
    }

    /// clears the set
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl<K: Hash + Eq, S: BuildHasher> LinkedHashSet<K, S> {
    /// returns size
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// returns the first element from the set
    pub fn pop_front(&mut self) -> Option<K> {
        match self.0.pop_front() {
            Some((k, _)) => Some(k),
            None => None
        }
    }

    /// returns the last element from the set
    pub fn pop_back(&mut self) -> Option<K> {
        match self.0.pop_back() {
            Some((k, _)) => Some(k),
            None => None
        }
    }

    /// inserts an element at the back
    pub fn insert(&mut self, k: K) -> Option<()> {
        self.0.insert(k, ())
    }

    /// returns true if the set contains the element, otherwise returns false
    pub fn contains<Q: ?Sized>(&self, k: &Q) -> bool
        where K: Borrow<Q>,
              Q: Eq + Hash
    {
        self.0.contains_key(k)
    }

    /// removes an element from the set, do nothing if the set does not contain the element
    pub fn remove<Q: ?Sized>(&mut self, k: &Q)
        where K: Borrow<Q>,
              Q: Eq + Hash
    {
        self.0.remove(k);
    }

    /// gets an Keys iterator for the set
    pub fn iter(&self) -> Keys<K, ()> {
        self.0.keys()
    }

    /// returns true if the set is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// pops all elements from the other LinkedHashSet, and adds to this LinkedHashSet
    pub fn add_all(&mut self, mut other: Self) {
        while !other.is_empty() {
            let entry = other.pop_front().unwrap();
            self.insert(entry);
        }
    }

    /// pops all elements from a vector, and adds to this LinkedHashSet
    pub fn add_from_vec(&mut self, mut vec: Vec<K>) {
        while !vec.is_empty() {
            self.insert(vec.pop().unwrap());
        }
    }

    /// returns true if two LinkedHashSets have same elements (ignore order)
    pub fn equals(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for ele in self.iter() {
            if !other.contains(ele) {
                return false;
            }
        }

        true
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
