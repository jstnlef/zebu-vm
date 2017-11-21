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

use utils::ByteSize;

pub mod sidemap;
pub use self::sidemap::*;

pub mod immortal;
pub use self::immortal::*;

pub fn init() {
    use objectmodel::sidemap::*;
    GlobalTypeTable::init();
}

pub fn cleanup() {
    use objectmodel::sidemap::*;
    GlobalTypeTable::cleanup();
}

#[inline(always)]
pub fn check_alignment(align: ByteSize) -> ByteSize {
    if align < MINIMAL_ALIGNMENT {
        MINIMAL_ALIGNMENT
    } else {
        align
    }
}

#[inline(always)]
pub fn check_size(size: ByteSize) -> ByteSize {
    if size < MINIMAL_OBJECT_SIZE {
        MINIMAL_OBJECT_SIZE
    } else {
        size
    }
}
