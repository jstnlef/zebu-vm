mod malloc_list;
mod treadmill;

//pub use heap::freelist::malloc_list::FreeListSpace;
pub use heap::freelist::treadmill::FreeListSpace;

use std::sync::Arc;
use std::sync::RwLock;
use heap::gc;
use utils::{Address, ObjectReference};
use heap::immix;

#[inline(never)]
pub fn alloc_large(size: usize, align: usize, mutator: &mut immix::ImmixMutatorLocal, space: Arc<FreeListSpace>) -> Address {
    loop {
        mutator.yieldpoint();

        let ret_addr = space.alloc(size, align);

        if ret_addr.is_zero() {
            gc::trigger_gc();
        } else {
            return ret_addr;
        }
    }
}