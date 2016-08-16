extern crate mu;
extern crate log;
extern crate simple_logger;

mod test_ir;
mod test_compiler;
mod test_vm;
mod test_api;

#[macro_use]
mod common {
    use std::fmt;
    
    pub fn assert_vector_ordered <T: fmt::Debug> (left: &Vec<T>, right: &Vec<T>) {
        assert_debug_str(left, right);
    }
    
    pub fn assert_vector_no_order <T: Ord + fmt::Debug + Clone> (left: &Vec<T>, right: &Vec<T>) {
        let mut left_clone = left.clone();
        left_clone.sort();
        let mut right_clone = right.clone();
        right_clone.sort();
        
        assert_debug_str(left_clone, right_clone);
    }
    
    pub fn assert_debug_str<T: fmt::Debug, U: fmt::Debug> (left: T, right: U) {
        assert_eq!(format!("{:?}", left), format!("{:?}", right))
    }
}
