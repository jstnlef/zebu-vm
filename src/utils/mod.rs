// This porvides some missing operations on Vec.
// They are not included in the standard libarary. 
// (because they are likely inefficient?)
pub mod vec_utils {
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

pub mod hashset_utils {
    use std::collections::HashSet;
    use std::hash::Hash;
    
    pub fn pop_first<T: Eq + Hash + Copy> (set: &mut HashSet<T>) -> Option<T> {
        if set.is_empty() {
            None
        } else {
            let next : T = set.iter().next().unwrap().clone();
            set.take(&next)
        }
    }
}