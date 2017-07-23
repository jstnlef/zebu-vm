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

use std::sync::atomic;
use utils::ByteSize;

#[cfg(feature = "use-sidemap")]
mod sidemap;
#[cfg(not(feature = "use-sidemap"))]
mod header;

// mark state

pub static INIT_MARK_STATE: usize = 1;
static MARK_STATE: atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

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

#[inline(always)]
pub fn check_alignment(align: ByteSize) -> ByteSize {
    if align < MINIMAL_ALIGNMENT {
        MINIMAL_ALIGNMENT
    } else {
        align
    }
}

// --- sidemap object model ---

#[cfg(feature = "use-sidemap")]
pub use self::sidemap::gen_gctype_encode;
#[cfg(feature = "use-sidemap")]
pub use self::sidemap::gen_hybrid_gctype_encode;

#[cfg(feature = "use-sidemap")]
pub use self::sidemap::MINIMAL_ALIGNMENT;
#[cfg(feature = "use-sidemap")]
pub use self::sidemap::OBJECT_HEADER_SIZE;
#[cfg(feature = "use-sidemap")]
pub use self::sidemap::OBJECT_HEADER_OFFSET;
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

// --- header ----

#[cfg(not(feature = "use-sidemap"))]
pub use self::header::gen_gctype_encode;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::gen_hybrid_gctype_encode;

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
pub use self::header::REF_MAP_LENGTH;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::SHR_HYBRID_LENGTH;

// header location/size
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::MINIMAL_ALIGNMENT;
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
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::header_get_object_size;
#[cfg(not(feature = "use-sidemap"))]
pub use self::header::header_get_hybrid_length;
