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

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use std::mem;
use utils::mem::memmap;
use utils::math;
use utils::Address;

use objectmodel::sidemap::TypeID;
use objectmodel::sidemap::N_TYPES;
use objectmodel::sidemap::type_encode::TypeEncode;
use objectmodel::sidemap::object_encode::SMALL_ID_WIDTH;

/// represents a chunk of memory as global type table, which contains some metadata for the
/// type table and all the type encoding entries.
///
/// The memory looks like this
///
/// |----------------|
/// | metadata(this) |
/// | ...            |
/// |----------------| <- global_type_table points to this
/// |                |    at next 128 bytes alignment (size of TypeEncoding)
/// | small entries  |
/// | ...            |
/// | ...            |
/// |----------------| (8192 entries = 1 << 13 (SMALL_ID_WIDTH) )
/// | large entries  |
/// | ...            |
/// | ...            |
/// |________________|
///
#[repr(C, packed)]
pub struct GlobalTypeTable {
    /// current index for small entries
    small_entry_i: usize,
    /// current index for large entries
    large_entry_i: usize
}

const SMALL_ENTRY_CAP: usize = 1 << SMALL_ID_WIDTH;
const LARGE_ENTRY_CAP: usize = N_TYPES;

/// storing a pointer to the actual type table
static global_type_table_ptr: AtomicUsize = ATOMIC_USIZE_INIT;
/// storing a pointer to the metadata of the type table
static global_type_table_meta: AtomicUsize = ATOMIC_USIZE_INIT;
/// save Mmap to keep the memory map alive
//  it is okay to use lock here, as we won't actually access this field
lazy_static!{
    static ref gtt_mmap: Mutex<Option<memmap::Mmap>> = Mutex::new(None);
}

impl GlobalTypeTable {
    pub fn init() {
        let mut mmap_lock = gtt_mmap.lock().unwrap();
        assert!(mmap_lock.is_none());

        let entry_size = mem::size_of::<TypeEncode>();
        let metadata_size = math::align_up(mem::size_of::<GlobalTypeTable>(), entry_size);

        let mmap = match memmap::Mmap::anonymous(
            metadata_size + N_TYPES * entry_size,
            memmap::Protection::ReadWrite
        ) {
            Ok(m) => m,
            Err(_) => panic!("failed to mmap for global type table")
        };

        info!("Global Type Table allocated at {:?}", mmap.ptr());

        // start address of metadata
        let meta_addr = Address::from_ptr::<u8>(mmap.ptr());
        global_type_table_meta.store(meta_addr.as_usize(), Ordering::Relaxed);
        // actual table
        let table_addr = meta_addr + metadata_size;
        global_type_table_ptr.store(table_addr.as_usize(), Ordering::Relaxed);

        // initialize meta
        let meta: &mut GlobalTypeTable = unsafe { meta_addr.to_ptr_mut().as_mut().unwrap() };
        meta.small_entry_i = 0;
        meta.large_entry_i = SMALL_ENTRY_CAP;

        // save mmap
        *mmap_lock = Some(mmap);
        trace!("Global Type Table initialization done");
    }

    #[inline(always)]
    fn table_meta() -> &'static mut GlobalTypeTable {
        unsafe { mem::transmute(global_type_table_meta.load(Ordering::Relaxed)) }
    }

    #[inline(always)]
    pub fn table() -> &'static mut [TypeEncode; N_TYPES] {
        unsafe { mem::transmute(global_type_table_ptr.load(Ordering::Relaxed)) }
    }

    pub fn insert_small_entry(entry: TypeEncode) -> TypeID {
        let mut meta = GlobalTypeTable::table_meta();
        let mut table = GlobalTypeTable::table();

        if meta.small_entry_i < SMALL_ENTRY_CAP {
            let id = meta.small_entry_i;
            table[id] = entry;
            meta.small_entry_i += 1;
            id
        } else {
            panic!("small type entries overflow the global type table")
        }
    }

    pub fn insert_large_entry(entry: TypeEncode) -> TypeID {
        let mut meta = GlobalTypeTable::table_meta();
        let mut table = GlobalTypeTable::table();

        if meta.large_entry_i < LARGE_ENTRY_CAP {
            let id = meta.large_entry_i;
            table[id] = entry;
            meta.large_entry_i += 1;
            id
        } else {
            panic!("large type entries overflow the global type table")
        }
    }
}

#[cfg(test)]
mod global_type_table_test {
    use super::*;
    use objectmodel::sidemap::type_encode::WordType::*;
    use start_logging_trace;

    #[test]
    fn test_insert() {
        start_logging_trace();

        GlobalTypeTable::init();
        let ty = {
            let mut fix_ty = [0; 63];
            fix_ty[0] = 0b11100100u8;
            fix_ty[1] = 0b00011011u8;
            fix_ty[2] = 0b11100100u8;
            TypeEncode::new(12, fix_ty, 0, [0; 63])
        };
        let tyid = GlobalTypeTable::insert_small_entry(ty) as usize;

        let ref loaded_ty = GlobalTypeTable::table()[tyid];
        assert_eq!(loaded_ty.fix_ty(0), NonRef);
        assert_eq!(loaded_ty.fix_ty(1), Ref);
        assert_eq!(loaded_ty.fix_ty(2), WeakRef);
        assert_eq!(loaded_ty.fix_ty(3), TaggedRef);
        assert_eq!(loaded_ty.fix_ty(4), TaggedRef);
        assert_eq!(loaded_ty.fix_ty(5), WeakRef);
        assert_eq!(loaded_ty.fix_ty(6), Ref);
        assert_eq!(loaded_ty.fix_ty(7), NonRef);
        assert_eq!(loaded_ty.fix_ty(8), NonRef);
        assert_eq!(loaded_ty.fix_ty(9), Ref);
        assert_eq!(loaded_ty.fix_ty(10), WeakRef);
        assert_eq!(loaded_ty.fix_ty(11), TaggedRef);
    }
}
