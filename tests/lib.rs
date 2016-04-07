mod test_ir;
mod test_compiler;

#[macro_use]
mod common {
    use std::fmt;
    
    pub fn assert_str_vector (left: &Vec<&str>, right: &Vec<&str>) {
        left.clone().sort();
        right.clone().sort();
        
        assert_debug_str(left, right);
    }
    
    pub fn assert_debug_str<T: fmt::Debug, U: fmt::Debug> (left: T, right: U) {
        assert_eq!(format!("{:?}", left), format!("{:?}", right))
    }
}