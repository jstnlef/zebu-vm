use utils::Address;
use utils::bit_utils;
use utils::POINTER_SIZE;
use utils::LOG_POINTER_SIZE;
use std::sync::atomic::AtomicUsize;

use objectmodel;

pub mod immix;
pub mod freelist;
pub mod gc;

pub const ALIGNMENT_VALUE : u8 = 1;

pub const IMMIX_SPACE_RATIO : f64 = 1.0 - LO_SPACE_RATIO;
pub const LO_SPACE_RATIO : f64 = 0.2;
pub const DEFAULT_HEAP_SIZE : usize = 500 << 20;

lazy_static! {
    pub static ref IMMIX_SPACE_SIZE : AtomicUsize = AtomicUsize::new( (DEFAULT_HEAP_SIZE as f64 * IMMIX_SPACE_RATIO) as usize );
    pub static ref LO_SPACE_SIZE : AtomicUsize = AtomicUsize::new( (DEFAULT_HEAP_SIZE as f64 * LO_SPACE_RATIO) as usize );
}

pub trait Space {
    #[inline(always)]
    fn start(&self) -> Address;
    #[inline(always)]
    fn end(&self) -> Address;

    #[inline(always)]
    fn alloc_map(&self) -> *mut u8;
    #[inline(always)]
    fn trace_map(&self) -> *mut u8;

    #[inline(always)]
    fn is_valid_object(&self, addr: Address) -> bool {
        let start = self.start();
        let end = self.end();

        if addr >= end || addr < start {
            return false;
        }

        let index = (addr.diff(start) >> LOG_POINTER_SIZE) as isize;

        if !bit_utils::test_nth_bit(unsafe {*self.alloc_map().offset(index)}, objectmodel::OBJ_START_BIT) {
            return false;
        }

        if !addr.is_aligned_to(POINTER_SIZE) {
            return false;
        }

        true
    }

    #[inline(always)]
    fn addr_in_space(&self, addr: Address) -> bool {
        addr >= self.start() && addr < self.end()
    }
}

#[inline(always)]
pub fn fill_alignment_gap(start : Address, end : Address) -> () {
    debug_assert!(end >= start);
    start.memset(ALIGNMENT_VALUE, end.diff(start));
}