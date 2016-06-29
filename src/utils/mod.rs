#![allow(dead_code)]

mod linked_hashset;
pub use utils::linked_hashset::LinkedHashSet;
pub use utils::linked_hashset::LinkedHashMap;

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
