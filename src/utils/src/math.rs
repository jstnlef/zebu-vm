pub fn is_power_of_two(x: usize) -> Option<u8> {
    use std::u8;

    let mut power_of_two = 1;
    let mut i : u8 = 0;
    while power_of_two < x && i < u8::MAX {
        power_of_two *= 2;
        i += 1;
    }

    if power_of_two == x {
        Some(i)
    } else {
        None
    }
}

pub fn align_up(x: usize, align: usize) -> usize {
    if x % align == 0 {
        x
    } else {
        (x / align + 1) * align
    }
}