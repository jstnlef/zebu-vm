#![allow(dead_code)]
pub type ByteSize = usize;

pub mod mem;
mod linked_hashset;
pub use utils::linked_hashset::LinkedHashSet;
pub use utils::linked_hashset::LinkedHashMap;

pub use runtime::Address;
pub use runtime::ObjectReference;

macro_rules! select_value {
    ($cond: expr, $res1 : expr, $res2 : expr) => {
        if $cond {
            $res1
        } else {
            $res2
        }
    }
}

// This porvides some missing operations on Vec.
// They are not included in the standard libarary.
// (because they are likely inefficient?)
pub mod vec_utils {
    use std::fmt;
    
    pub fn is_identical_to_str_ignore_order<T: Ord + fmt::Display + Clone, Q: Ord + fmt::Display + Clone> (vec: &Vec<T>, mut expect: Vec<Q>) -> bool {
        let mut vec_copy = vec.to_vec();
        vec_copy.sort();
        
        expect.sort();
        
        let a = as_str(&vec_copy);
        let b = as_str(&expect);
        
        a == b
    }
    
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

    pub fn add_all<T: Copy + PartialEq> (vec: &mut Vec<T>, vec2: &Vec<T>) -> bool {
        let mut is_changed = false;

        for i in vec2.iter() {
            if !vec.contains(i) {
                vec.push(*i);
                is_changed = true;
            }
        }

        is_changed
    }
    
    pub fn add_unique<T: PartialEq> (vec: &mut Vec<T>, val: T) {
        if !vec.contains(&val) {
            vec.push(val);
        }
    }
    
    pub fn append_unique<T: PartialEq> (vec: &mut Vec<T>, vec2: &mut Vec<T>) {
        while !vec2.is_empty() {
            let val = vec2.pop().unwrap();
            add_unique(vec, val);
        }
    }

    pub fn find_value<T: PartialEq> (vec: &Vec<T>, val: T) -> Option<usize> {
        for i in 0..vec.len() {
            if vec[i] == val {
                return Some(i);
            }
        }

        None
    }

    pub fn remove_value<T: PartialEq> (vec: &mut Vec<T>, val: T) {
        match find_value(vec, val) {
            Some(index) => {vec.remove(index);},
            None => {} // do nothing
        }
    }
    
    pub fn map<T, Q, F> (vec: &Vec<T>, map_func: F) -> Vec<Q> 
        where F : Fn(&T) -> Q {
        let mut ret = vec![];
        
        for t in vec {
            ret.push(map_func(t));
        }
        
        ret
    }
}

pub mod string_utils {
   pub fn replace(s: &mut String, index: usize, replace: &String, replace_len: usize) {
        let vec = unsafe {s.as_mut_vec()};
        let vec_replace = replace.as_bytes();

        for i in 0..replace_len {
            if i < replace.len() {
                vec[index + i] = vec_replace[i] as u8;
            } else {
                vec[index + i] = ' ' as u8;
            }
        }
    }
}
