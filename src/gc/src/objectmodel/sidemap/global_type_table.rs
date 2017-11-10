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
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use std::collections::HashMap;
use std::mem;
use utils::mem::memmap;
use utils::math;
use utils::Address;

use objectmodel::sidemap::TypeID;
use objectmodel::sidemap::N_TYPES;
use objectmodel::sidemap::type_encode::*;
use objectmodel::sidemap::object_encode::SMALL_ID_WIDTH;

/// represents a chunk of memory as global type table, which contains some metadata for the
/// type table and all the type encoding entries.
#[repr(C, packed)]
pub struct GlobalTypeTable {
    /// current index for small entries
    small_entry_i: usize,
    /// current index for large entries
    large_entry_i: usize,
    /// full entries
    full_entry_i: usize,
    full_entries: RwLock<HashMap<usize, FullTypeEncode>>,

    #[allow(dead_code)]
    mmap: memmap::MmapMut,

    table: [ShortTypeEncode; N_TYPES]
}

const SMALL_ENTRY_CAP: usize = 1 << SMALL_ID_WIDTH;
const LARGE_ENTRY_CAP: usize = N_TYPES;
const FULL_ENTRY_START: usize = LARGE_ENTRY_CAP + 1;

/// storing a pointer to the actual type table
static GLOBAL_TYPE_TABLE_PTR: AtomicUsize = ATOMIC_USIZE_INIT;
/// storing a pointer to the metadata of the type table
static GLOBAL_TYPE_TABLE_META: AtomicUsize = ATOMIC_USIZE_INIT;

impl GlobalTypeTable {
    pub fn init() {
        debug!("Init GlobalTypeTable...");
        let mut mmap = match memmap::MmapMut::map_anon(mem::size_of::<GlobalTypeTable>()) {
            Ok(m) => m,
            Err(_) => panic!("failed to mmap for global type table")
        };

        info!("Global Type Table allocated at {:?}", mmap.as_mut_ptr());

        // start address of metadata
        let meta_addr = Address::from_ptr::<u8>(mmap.as_mut_ptr());
        GLOBAL_TYPE_TABLE_META.store(meta_addr.as_usize(), Ordering::Relaxed);

        let mut meta: &mut GlobalTypeTable = unsafe { meta_addr.to_ref_mut() };

        // actual table
        let table_addr = Address::from_ptr(&meta.table as *const [ShortTypeEncode; N_TYPES]);
        GLOBAL_TYPE_TABLE_PTR.store(table_addr.as_usize(), Ordering::Relaxed);

        // initialize meta
        meta.small_entry_i = 0;
        meta.large_entry_i = SMALL_ENTRY_CAP;
        meta.full_entry_i = FULL_ENTRY_START;
        unsafe {
            use std::ptr;
            ptr::write(
                &mut meta.full_entries as *mut RwLock<HashMap<usize, FullTypeEncode>>,
                RwLock::new(HashMap::new())
            )
        }
        unsafe {
            use std::ptr;
            ptr::write(&mut meta.mmap as *mut memmap::MmapMut, mmap);
        }

        // save mmap
        trace!("Global Type Table initialization done");
    }

    pub fn cleanup() {
        GLOBAL_TYPE_TABLE_PTR.store(0, Ordering::Relaxed);
        GLOBAL_TYPE_TABLE_META.store(0, Ordering::Relaxed);
    }

    #[inline(always)]
    fn table_meta() -> &'static mut GlobalTypeTable {
        unsafe { mem::transmute(GLOBAL_TYPE_TABLE_META.load(Ordering::Relaxed)) }
    }

    #[inline(always)]
    pub fn table() -> &'static mut [ShortTypeEncode; N_TYPES] {
        unsafe { mem::transmute(GLOBAL_TYPE_TABLE_PTR.load(Ordering::Relaxed)) }
    }

    pub fn insert_small_entry(entry: ShortTypeEncode) -> TypeID {
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

    pub fn insert_large_entry(entry: ShortTypeEncode) -> TypeID {
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

    pub fn force_set_short_entry(index: TypeID, entry: ShortTypeEncode) {
        let mut meta = GlobalTypeTable::table_meta();
        let mut table = GlobalTypeTable::table();

        table[index] = entry;

        if index < SMALL_ENTRY_CAP {
            if meta.small_entry_i < index {
                meta.small_entry_i = index;
            }
        } else if index < LARGE_ENTRY_CAP {
            if meta.large_entry_i < index {
                meta.large_entry_i = index;
            }
        } else {
            panic!(
                "TypeID {} exceeds LARGE_ENTRY_CAP, try use insert_full_entry() instead",
                index
            )
        }
    }

    pub fn insert_full_entry(entry: FullTypeEncode) -> TypeID {
        let meta = GlobalTypeTable::table_meta();

        let mut lock = meta.full_entries.write().unwrap();
        let id = meta.full_entry_i;
        lock.insert(id, entry);
        meta.full_entry_i += 1;

        id
    }

    pub fn force_set_full_entry(index: TypeID, entry: FullTypeEncode) {
        let mut meta = GlobalTypeTable::table_meta();
        let mut lock = meta.full_entries.write().unwrap();
        assert!(!lock.contains_key(&index));
        lock.insert(index, entry);

        if meta.full_entry_i < index {
            meta.full_entry_i = index;
        }
    }

    pub fn get_full_type(id: usize) -> FullTypeEncode {
        let meta = GlobalTypeTable::table_meta();
        let lock = meta.full_entries.read().unwrap();
        debug_assert!(lock.contains_key(&id));
        lock.get(&id).unwrap().clone()
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
            ShortTypeEncode::new(8, 12, fix_ty, 0, [0; 63])
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
