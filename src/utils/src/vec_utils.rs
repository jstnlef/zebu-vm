    use std::fmt;
    
    pub fn is_identical_to_str_ignore_order<T: Ord + fmt::Display + Clone, Q: Ord + fmt::Display + Clone> (vec: &Vec<T>, mut expect: Vec<Q>) -> bool {
        let mut vec_copy = vec.to_vec();
        vec_copy.sort();
        
        expect.sort();
        
        let a = as_str(&vec_copy);
        let b = as_str(&expect);
        
        a == b
    }
    
    pub fn is_identical_ignore_order<T: Ord + Clone> (vec: &Vec<T>, vec2: &Vec<T>) -> bool {
        if vec.len() != vec2.len() {
            return false;
        }
        
        let mut vec = vec.to_vec();
        let mut vec2 = vec2.to_vec();
        vec.sort();
        vec2.sort();
        
        for i in 0..vec.len() {
            if vec[i] != vec2[i] {
                return false;
            }
        }
        
        return true;
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
    
    pub fn append_clone_unique<T: PartialEq + Clone> (vec: &mut Vec<T>, vec2: &Vec<T>) {
        for ele in vec2 {
            add_unique(vec, ele.clone());
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