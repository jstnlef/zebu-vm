#[inline(always)]
pub fn test_nth_bit(value: u8, index: usize) -> bool {
    value & (1 << index) != 0
}

#[inline(always)]
pub fn lower_bits(value: u8, len: usize) -> u8 {
    value & ((1 << len) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    pub fn test_u8_bits() {
        let value : u8 = 0b1100_0011;
        
        assert_eq!(test_nth_bit(value, 6), true);
        
        assert_eq!(lower_bits(value, 6), 0b00_0011);
    }
}