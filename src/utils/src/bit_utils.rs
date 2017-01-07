// u8

#[inline(always)]
pub fn test_nth_bit_u8(value: u8, index: usize, val: u8) -> bool {
    ((value >> index) & 1) as u8 == val
}

#[inline(always)]
pub fn lower_bits_u8(value: u8, len: usize) -> u8 {
    value & ((1 << len) - 1)
}

// u64

#[inline(always)]
pub fn set_nth_bit_u64 (value: u64, index: usize, set_value: u8) -> u64 {
    value ^ (((-(set_value as i64) as u64) ^ value) & (1 << index))
}

#[inline(always)]
pub fn test_nth_bit_u64(value: u64, index: usize, val: u8) -> bool {
    ((value >> index) & 1) as u8 == val
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    pub fn test_u8_bits() {
        let value : u8 = 0b1100_0011;
        
        assert_eq!(test_nth_bit_u8(value, 6), true);
        
        assert_eq!(lower_bits_u8(value, 6), 0b00_0011);
    }

    #[test]
    pub fn test_set_bit() {
        let a = 0b0000u64;
        let b = 0b1111u64;

        assert_eq!(set_nth_bit_u64(a, 2, 1), 0b100);
        assert_eq!(set_nth_bit_u64(b, 2, 0), 0b1011);
    }
}
