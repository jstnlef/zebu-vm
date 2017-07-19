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

use std::fmt;

/// returns a formatted String for a Vec<T> (T needs Display trait)
pub fn as_str<T: fmt::Display>(vec: &Vec<T>) -> String {
    let mut ret = String::new();
    for i in 0..vec.len() {
        ret.push_str(format!("{}", vec[i]).as_str());
        if i != vec.len() - 1 {
            ret.push_str(", ");
        }
    }
    ret
}

/// adds a value to the vector if the vector does not contains a same value
pub fn add_unique<T: PartialEq>(vec: &mut Vec<T>, val: T) {
    if !vec.contains(&val) {
        vec.push(val);
    }
}

/// adds all values from 2nd vector to the first one if the vector does not contains a same value
/// This function will pop all elements from the 2nd vector.
pub fn add_all_unique<T: PartialEq>(vec: &mut Vec<T>, vec2: &mut Vec<T>) {
    while !vec2.is_empty() {
        let val = vec2.pop().unwrap();
        add_unique(vec, val);
    }
}

/// returns the index of a given value in the vector
/// returns None if the vector does not contains the value
pub fn find_value<T: PartialEq>(vec: &Vec<T>, val: T) -> Option<usize> {
    for i in 0..vec.len() {
        if vec[i] == val {
            return Some(i);
        }
    }

    None
}

/// intersects 1st vector with the 2nd one
/// (retains elements that also appear in the 2nd vector, and deletes elements that do not)
pub fn intersect<T: PartialEq + Clone>(vec: &mut Vec<T>, vec2: &Vec<T>) -> bool {
    let mut indices_to_delete = vec![];

    for i in 0..vec.len() {
        if find_value(vec2, vec[0].clone()).is_none() {
            indices_to_delete.push(i);
        }
    }

    for i in indices_to_delete.iter() {
        vec.remove(*i);
    }

    indices_to_delete.len() != 0
}

/// removes a value from a vector (if there are several appearances, delete the first one)
pub fn remove_value<T: PartialEq>(vec: &mut Vec<T>, val: T) {
    match find_value(vec, val) {
        Some(index) => {
            vec.remove(index);
        }
        None => {} // do nothing
    }
}

/// maps each element in the vector with a map function, and returns the new vector
pub fn map<T, Q, F>(vec: &Vec<T>, map_func: F) -> Vec<Q>
where
    F: Fn(&T) -> Q,
{
    let mut ret = vec![];

    for t in vec {
        ret.push(map_func(t));
    }

    ret
}
