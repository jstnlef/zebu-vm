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
use utils::ByteOffset;

mod immix_space;
mod immix_mutator;

pub use self::immix_space::ImmixSpace;
pub use self::immix_space::ImmixBlock;
pub use self::immix_mutator::ImmixAllocator;
pub use self::immix_mutator::CURSOR_OFFSET;
pub use self::immix_mutator::LIMIT_OFFSET;

pub use self::immix_space::mark_object_traced;
pub use self::immix_space::is_object_traced;

pub const LOG_BYTES_IN_LINE: usize = 8;
pub const BYTES_IN_LINE: ByteSize = (1 << LOG_BYTES_IN_LINE);

pub const LOG_BYTES_IN_BLOCK: usize = 16;
pub const BYTES_IN_BLOCK: ByteSize = (1 << LOG_BYTES_IN_BLOCK);
/// size of metadata for block (should be the same as size_of::<ImmixBlock>())
pub const BLOCK_META: ByteSize = 16;
/// GC map immediately follows the meta data
pub const OFFSET_GC_MAP_IN_BLOCK: ByteOffset = BLOCK_META as ByteOffset;
/// GC map byte size
pub const BYTES_GC_MAP_IN_BLOCK: ByteSize = (BYTES_IN_BLOCK - BLOCK_META) / 9 / 2;
/// type map immediately follows the GC map
pub const OFFSET_TYPE_MAP_IN_BLOCK: ByteOffset =
    OFFSET_GC_MAP_IN_BLOCK + BYTES_GC_MAP_IN_BLOCK as isize;
/// type map byte size
pub const BYTES_TYPE_MAP_IN_BLOCK: ByteSize = BYTES_GC_MAP_IN_BLOCK;
/// the memory start for actual use
pub const OFFSET_MEMORY_START_IN_BLOCK: ByteOffset =
    OFFSET_TYPE_MAP_IN_BLOCK + BYTES_TYPE_MAP_IN_BLOCK as isize;
/// size of usable memory in a block
pub const BYTES_MEM_IN_BLOCK: ByteSize =
    BYTES_IN_BLOCK - BLOCK_META - BYTES_GC_MAP_IN_BLOCK - BYTES_TYPE_MAP_IN_BLOCK;
/// how many lines are in block (227)
pub const LINES_IN_BLOCK: usize =
    (BYTES_IN_BLOCK - BLOCK_META - BYTES_GC_MAP_IN_BLOCK - BYTES_TYPE_MAP_IN_BLOCK) / BYTES_IN_LINE;

pub const IMMIX_SPACE_ALIGN: ByteSize = (1 << 19); // 512K
pub const IMMIX_SPACE_LOWBITS_MASK: usize = !(IMMIX_SPACE_ALIGN - 1);

pub const IMMIX_BLOCK_ALIGN: ByteSize = BYTES_IN_BLOCK; // 64K
pub const IMMIX_BLOCK_LOWBITS_MASK: usize = !(IMMIX_BLOCK_ALIGN - 1);

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum LineMark {
    Free = 0,
    Live,
    FreshAlloc,
    ConservLive,
    PrevLive
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BlockMark {
    Uninitialized,
    Usable,
    Full
}
