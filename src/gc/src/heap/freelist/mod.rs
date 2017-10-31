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
use utils::Address;
use heap::gc;
use heap::Mutator;

mod malloc_list;
mod treadmill;

//pub use heap::freelist::malloc_list::FreeListSpace;
pub use heap::freelist::treadmill::FreeListSpace;

use std::sync::Arc;

#[inline(never)]
pub fn alloc_large(
    size: usize,
    align: usize,
    mutator: &mut Mutator,
    space: Arc<FreeListSpace>
) -> Address {
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
