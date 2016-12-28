use std::sync::atomic;
use utils::{Address, ObjectReference};
use utils::{LOG_POINTER_SIZE, POINTER_SIZE};
use utils::bit_utils;

mod sidemap;
mod header;

// mark state

pub static INIT_MARK_STATE : usize = 1;
static MARK_STATE : atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

pub fn init() {
    MARK_STATE.store(INIT_MARK_STATE, atomic::Ordering::SeqCst);
}

pub fn flip_mark_state() {
    let mark_state = MARK_STATE.load(atomic::Ordering::SeqCst);
    MARK_STATE.store(mark_state ^ 1, atomic::Ordering::SeqCst);
}

pub fn load_mark_state() -> u8 {
    MARK_STATE.load(atomic::Ordering::SeqCst) as u8
}

pub fn flip(mark: u8) -> u8 {
    mark ^ 1
}

// sidemap object model

#[cfg(feature = "use-sidemap")]
pub use self::sidemap::OBJECT_HEADER_SIZE;
#[cfg(feature = "use-sidemap")]
pub use self::sidemap::REF_BITS_LEN;
#[cfg(feature = "use-sidemap")]
pub use self::sidemap::OBJ_START_BIT;
#[cfg(feature = "use-sidemap")]
pub use self::sidemap::SHORT_ENCODE_BIT;

#[cfg(feature = "use-sidemap")]
pub use self::sidemap::print_object;
#[cfg(feature = "use-sidemap")]
pub use self::sidemap::mark_as_traced;
#[cfg(feature = "use-sidemap")]
pub use self::sidemap::mark_as_untraced;
#[cfg(feature = "use-sidemap")]
pub use self::sidemap::is_traced;
#[cfg(feature = "use-sidemap")]
pub use self::sidemap::get_ref_byte;

// header

// flag bit
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::BIT_HAS_REF_MAP;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::BIT_IS_TRACED;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::BIT_IS_FIX_SIZE;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::BIT_IS_OBJ_START;

// field mask
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::MASK_GCTYPE_ID;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::MASK_HYBRID_LENGTH;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::MASK_REF_MAP;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::SHR_HYBRID_LENGTH;

// header location/size
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::OBJECT_HEADER_SIZE;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::OBJECT_HEADER_OFFSET;

#[cfg(not(feature = "use-sidemap"))]
pub use self::header::print_object;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::mark_as_traced;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::mark_as_untraced;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::is_traced;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::header_is_fix_size;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::header_has_ref_map;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::header_is_object_start;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::header_get_gctype_id;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::header_get_ref_map;

