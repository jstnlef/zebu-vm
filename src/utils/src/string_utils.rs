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
