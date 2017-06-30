// Copyright 2017 The Australian National University
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// returns true if the nth bit (count from least significant bit) is the same as val (as boolean)
/// otherwise returns false
#[inline(always)]
pub fn test_nth_bit_u8(value: u8, index: usize, val: u8) -> bool {
    ((value >> index) & 1) as u8 == val
}

/// returns the lower n bits
#[inline(always)]
pub fn lower_bits_u8(value: u8, len: usize) -> u8 {
    value & ((1 << len) - 1)
}

/// sets the nth bit (count from least significant bit) as val (treat the val as boolean, either 1 or 0)
#[inline(always)]
pub fn set_nth_bit_u64 (value: u64, index: usize, set_value: u8) -> u64 {
    value ^ (((-(set_value as i64) as u64) ^ value) & (1 << index))
}

/// returns true if the nth bit (count from least significant bit) is the same as val (as boolean)
/// otherwise returns false
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
        
        assert_eq!(test_nth_bit_u8(value, 6, 1), true);
        
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
