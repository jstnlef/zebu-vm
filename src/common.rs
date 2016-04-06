use std::fmt;

macro_rules! select_value {
    ($cond: expr, $res1 : expr, $res2 : expr) => {
        if $cond {
            $res1
        } else {
            $res2
        }
    }
}

pub fn vector_as_str<T: fmt::Display>(vec: &Vec<T>) -> String {
    let mut ret = String::new();
    for i in 0..vec.len() {
        ret.push_str(format!("{}", vec[i]).as_str());
        if i != vec.len() - 1 {
            ret.push_str(", ");
        }
    }
    ret
}