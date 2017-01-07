pub extern crate memmap;
pub extern crate memsec;

use Word;
#[allow(unused_imports)]
use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt, ByteOrder};

#[cfg(target_arch = "x86_64")]
pub fn u64_to_raw(val: u64) -> Word {
    let mut ret = vec![];
    ret.write_u64::<LittleEndian>(val).unwrap();
    
    as_word(ret)
}

#[cfg(target_arch = "x86_64")]
pub fn f32_to_raw(val: f32) -> Word {
    let mut ret = vec![];
    ret.write_f32::<LittleEndian>(val).unwrap();
    as_word(ret)
}

#[cfg(target_arch = "x86_64")]
pub fn f64_to_raw(val: f64) -> Word {
    let mut ret = vec![];
    ret.write_f64::<LittleEndian>(val).unwrap();
    as_word(ret)
}

#[cfg(target_arch = "x86_64")]
pub fn as_word(mut u8_array: Vec<u8>) -> Word {
    LittleEndian::read_uint(&mut u8_array, 8) as Word
}

#[cfg(test)]
mod tests{
    use super::*;
    use Word;
    
    #[test]
    fn test_primitive_to_raw() {
        let a : Word = 0xabcd;
        let raw = u64_to_raw(a as u64);
        
        assert_eq!(raw, a);
    }
}
