#![allow(dead_code)]

/// * use 1 word  (64bits)  header

/// * header is before an object reference

/// * for fix-sized types
///   MSB 1 bit - is object start
///       1 bit - trace bit
///       1 bit - is fix-sized (set for fix-sized types)
///       1 bit - is reference map encoded?
///       ... (unused)
///       16 bits -

/// fix-sized with reference map
/// | start? | trace? | fix? | ref map? | (unused bits)  ... | reference map (32bits) |
///                      1        1

/// fix-sized with ID
/// | start? | trace? | fix? | ref map? | (unused bits)  ... | gc type ID    (32bits) |
///                      1        0

/// var-sized
/// | start? | trace? | fix? | hybrid length (29bits ~ 500M) | gc type ID    (32bits) |
///                      0

use common::gctype::GCType;
use objectmodel;
use utils::ByteSize;
use utils::ByteOffset;
use utils::bit_utils;

use utils::{Address, ObjectReference};
use utils::POINTER_SIZE;
use utils::LOG_POINTER_SIZE;

pub const OBJECT_HEADER_SIZE   : ByteSize   = 8;
pub const OBJECT_HEADER_OFFSET : ByteOffset = - (OBJECT_HEADER_SIZE as ByteOffset);

pub const BIT_IS_OBJ_START  : usize = 63;
pub const BIT_IS_TRACED     : usize = 62;
pub const BIT_IS_FIX_SIZE   : usize = 61;
pub const BIT_HAS_REF_MAP   : usize = 60;

pub const REF_MAP_LENGTH    : usize = 32;
pub const MASK_REF_MAP      : u64 = 0xFFFFFFFFu64;
pub const MASK_GCTYPE_ID    : u64 = 0xFFFFFFFFu64;
pub const MASK_HYBRID_LENGTH: u64 = 0x1FFFFFFF00000000u64;
pub const SHR_HYBRID_LENGTH : usize = 32;

pub fn gen_gctype_encode(ty: &GCType) -> u64 {
    let mut ret = 0u64;

    if ty.repeat_refs.is_some() {
        // var sized
        let len = ty.repeat_refs.as_ref().unwrap().count;

        // encode length
        ret = ret | (( (len as u64) << SHR_HYBRID_LENGTH) & MASK_HYBRID_LENGTH);
        // encode gc id
        ret = ret | (ty.id as u64);
    } else {
        // fix sized
        ret = ret | (1 << BIT_IS_FIX_SIZE);

        // encode ref map?
        if ty.size < REF_MAP_LENGTH * POINTER_SIZE {
            // encode ref map
            let offsets = ty.gen_ref_offsets();
            let mut ref_map = 0;

            for offset in offsets {
                ref_map = ref_map | (1 << (offset >> LOG_POINTER_SIZE));
            }

            ret = ret | (ref_map & MASK_REF_MAP);
        } else {
            ret = ret | (ty.id as u64);
        }
    }

    ret
}

#[allow(unused_variables)]
pub fn print_object(obj: Address) {
    let mut cursor = obj;
    trace!("OBJECT 0x{:x}", obj);

    let hdr = unsafe {cursor.offset(OBJECT_HEADER_OFFSET).load::<u64>()};

    trace!("- is object start? {}", header_is_object_start(hdr));
    trace!("- is traced? {}", header_is_traced(hdr, objectmodel::load_mark_state()));
    if header_is_fix_size(hdr) {
        trace!("- is fix sized? true");
        if header_has_ref_map(hdr) {
            trace!("- has ref map: {:b}", header_get_ref_map(hdr));
        } else {
            trace!("- has type ID: {}", header_get_gctype_id(hdr));
        }
    } else {
        trace!("- more info about hybrid, not implemented");
    }

    trace!("0x{:x} | val: 0x{:15x} | hdr: {:b}",
    cursor, unsafe{cursor.load::<u64>()}, hdr);
    cursor = cursor.plus(POINTER_SIZE);
    trace!("0x{:x} | val: 0x{:15x}",
    cursor, unsafe{cursor.load::<u64>()});

    cursor = cursor.plus(POINTER_SIZE);
    trace!("0x{:x} | val: 0x{:15x}",
    cursor, unsafe{cursor.load::<u64>()});

    cursor = cursor.plus(POINTER_SIZE);
    trace!("0x{:x} | val: 0x{:15x}",
    cursor, unsafe{cursor.load::<u64>()});

    cursor = cursor.plus(POINTER_SIZE);
    trace!("0x{:x} | val: 0x{:15x}",
    cursor, unsafe{cursor.load::<u64>()});

    cursor = cursor.plus(POINTER_SIZE);
    trace!("0x{:x} | val: 0x{:15x}",
    cursor, unsafe{cursor.load::<u64>()});
}

#[inline(always)]
pub fn mark_as_traced(obj: ObjectReference, mark_state: u8) {
    unsafe {
        let hdr_addr = obj.to_address().offset(OBJECT_HEADER_OFFSET);
        hdr_addr.store(bit_utils::set_nth_bit_u64(hdr_addr.load::<u64>(), BIT_IS_TRACED, mark_state));
    }
}

#[inline(always)]
pub fn mark_as_untraced(addr: Address, mark_state: u8) {
    unsafe {
        let hdr_addr = addr.offset(OBJECT_HEADER_OFFSET);
        hdr_addr.store(bit_utils::set_nth_bit_u64(hdr_addr.load::<u64>(), BIT_IS_TRACED, mark_state ^ 1));
    }
}

#[inline(always)]
pub fn is_traced(obj: ObjectReference, mark_state: u8) -> bool {
    unsafe {
        let hdr = obj.to_address().offset(OBJECT_HEADER_OFFSET).load::<u64>();
        bit_utils::test_nth_bit_u64(hdr, BIT_IS_TRACED, mark_state)
    }
}

#[inline(always)]
pub fn header_is_object_start(hdr: u64) -> bool {
    bit_utils::test_nth_bit_u64(hdr, BIT_IS_OBJ_START, 1u8)
}

#[inline(always)]
pub fn header_is_fix_size(hdr: u64) -> bool {
    bit_utils::test_nth_bit_u64(hdr, BIT_IS_FIX_SIZE, 1u8)
}

#[inline(always)]
pub fn header_is_traced(hdr: u64, mark_state: u8) -> bool {
    bit_utils::test_nth_bit_u64(hdr, BIT_IS_TRACED, mark_state)
}

#[inline(always)]
pub fn header_has_ref_map(hdr: u64) -> bool {
    bit_utils::test_nth_bit_u64(hdr, BIT_HAS_REF_MAP, 1u8)
}

#[inline(always)]
pub fn header_get_ref_map(hdr: u64) -> u32 {
    (hdr & MASK_REF_MAP) as u32
}

#[inline(always)]
pub fn header_get_hybrid_length(hdr: u64) -> u32 {
    ((hdr & MASK_HYBRID_LENGTH) >> SHR_HYBRID_LENGTH) as u32
}

#[inline(always)]
pub fn header_get_gctype_id(hdr: u64) -> u32 {
    (hdr & MASK_GCTYPE_ID) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::gctype::*;
    use utils::POINTER_SIZE;
    use std::sync::Arc;

    #[test]
    fn fixsize_header_refmap() {
        let hdr_bin = 0b10110000_00000000_00000000_00000000_00000000_00000000_00000000_00000011u64;
        let hdr_hex = 0xb000000000000003u64;

        println!("");
        println!("binary: {:b}", hdr_bin);
        println!("hex   : {:b}", hdr_hex);

        assert_eq!(hdr_bin, hdr_hex);

        let hdr = hdr_hex;

        assert!(header_is_object_start(hdr));
        assert!(!header_is_traced(hdr, 1u8));
        assert!(header_is_fix_size(hdr));
        assert!(header_has_ref_map(hdr));

        assert_eq!(header_get_ref_map(hdr), 0b0011);
    }

    #[test]
    fn fixsize_header_id() {
        let hdr_bin = 0b10100000_00000000_00000000_00000000_00000000_00000000_00000000_11111111u64;
        let hdr_hex = 0xa0000000000000ffu64;

        println!("");
        println!("binary: {:b}", hdr_bin);
        println!("hex   : {:b}", hdr_hex);

        assert_eq!(hdr_bin, hdr_hex);

        let hdr = hdr_hex;

        assert!(header_is_object_start(hdr));
        assert!(!header_is_traced(hdr, 1u8));
        assert!(header_is_fix_size(hdr));
        assert!(!header_has_ref_map(hdr));

        assert_eq!(header_get_gctype_id(hdr), 0xff);
    }

    #[test]
    fn varsize_header() {
        let hdr_bin = 0b10000000_00000000_00000000_10000000_00000000_00000000_00000000_11111111u64;
        let hdr_hex = 0x80000080000000ffu64;

        println!("");
        println!("binary: {:b}", hdr_bin);
        println!("hex   : {:b}", hdr_hex);

        assert_eq!(hdr_bin, hdr_hex);

        let hdr = hdr_hex;

        assert!(header_is_object_start(hdr));
        assert!(!header_is_traced(hdr, 1u8));
        assert!(!header_is_fix_size(hdr));

        assert_eq!(header_get_hybrid_length(hdr), 128);
        assert_eq!(header_get_gctype_id(hdr), 0xff);
    }

    #[test]
    fn gctype_to_encode1() {
        // linked list: struct {ref, int64}
        let a = GCType{
            id: 0,
            size: 16,
            non_repeat_refs: Some(RefPattern::Map{
                offsets: vec![0],
                size: 16
            }),
            repeat_refs    : None
        };
        println!("gctype: {:?}", a);

        let encode = gen_gctype_encode(&a);
        println!("encode: {:64b}", encode);

        assert!(header_is_fix_size(encode));
        assert_eq!(header_get_ref_map(encode), 0b1);
    }

    #[test]
    fn gctype_to_encode2() {
        // doubly linked list: struct {ref, ref, int64, int64}
        let a = GCType{
            id: 0,
            size: 32,
            non_repeat_refs: Some(RefPattern::Map{
                offsets: vec![0, 8],
                size: 32
            }),
            repeat_refs    : None
        };
        println!("gctype: {:?}", a);

        let encode = gen_gctype_encode(&a);
        println!("encode: {:64b}", encode);

        assert!(header_is_fix_size(encode));
        assert_eq!(header_get_ref_map(encode), 0b11);
    }

    #[test]
    fn gctype_to_encode3() {
        // a struct of 64 references
        const N_REF : usize = 64;
        let a = GCType{
            id: 999,
            size: N_REF * POINTER_SIZE,
            non_repeat_refs: Some(RefPattern::Map{
                offsets: (0..N_REF).map(|x| x * POINTER_SIZE).collect(),
                size: N_REF * POINTER_SIZE
            }),
            repeat_refs    : None
        };
        println!("gctype: {:?}", a);

        let encode = gen_gctype_encode(&a);
        println!("encode: {:64b}", encode);

        assert!(header_is_fix_size(encode));
        assert_eq!(header_get_gctype_id(encode), 999);
    }

    #[test]
    fn gctype_to_encode4() {
        // array of struct {ref, int64} with length 10
        let a = GCType {
            id: 1,
            size: 160,
            non_repeat_refs: None,
            repeat_refs    : Some(RepeatingRefPattern {
                pattern: RefPattern::Map{
                    offsets: vec![0],
                    size   : 16
                },
                count  : 10
            }),
        };
        println!("gctype: {:?}", a);

        let encode = gen_gctype_encode(&a);
        println!("encode: {:64b}", encode);

        assert!(!header_is_fix_size(encode));
        assert_eq!(header_get_hybrid_length(encode), 10);
        assert_eq!(header_get_gctype_id(encode), 1);
    }

    #[test]
    fn gctype_to_encode5() {
        // array of struct {ref, int64} with length 10
        let b = GCType {
            id: 1,
            size: 160,
            non_repeat_refs: None,
            repeat_refs    : Some(RepeatingRefPattern {
                pattern: RefPattern::Map{
                    offsets: vec![0],
                    size   : 16
                },
                count  : 10
            }),
        };

        // array(10) of array(10) of struct {ref, int64}
        let a = GCType {
            id: 2,
            size: 1600,
            non_repeat_refs: None,
            repeat_refs    : Some(RepeatingRefPattern {
                pattern: RefPattern::NestedType(vec![Arc::new(b.clone()).clone()]),
                count  : 10
            })
        };
        println!("gctype: {:?}", a);

        let encode = gen_gctype_encode(&a);
        println!("encode: {:64b}", encode);

        assert!(!header_is_fix_size(encode));
        assert_eq!(header_get_hybrid_length(encode), 10);
        assert_eq!(header_get_gctype_id(encode), 2);
    }
}