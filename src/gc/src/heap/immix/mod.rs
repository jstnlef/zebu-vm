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

use utils::*;
use std::mem::size_of;

mod immix_space;
mod immix_mutator;

pub use self::immix_space::ImmixSpace;
pub use self::immix_space::ImmixBlock;
pub use self::immix_mutator::ImmixAllocator;
pub use self::immix_mutator::CURSOR_OFFSET;
pub use self::immix_mutator::LIMIT_OFFSET;

pub use self::immix_space::mark_object_traced;
pub use self::immix_space::is_object_traced;

// Immix space
// |------------------| <- 16GB align
// | metadata         |
// | ...              | (64 KB)
// |------------------|
// | block mark table | (256 KB) - 256K blocks, 1 byte per block
// |------------------|
// | line mark table  | (64MB) - 64M lines, 1 byte per line
// |------------------|
// | gc byte table    | (1GB) - 1/16 of memory, 1 byte per 16 (min alignment/object size)
// |------------------|
// | type byte table  | (1GB) - 1/16 of memory, 1 byte per 16 (min alignment/object size)
// |------------------|
// | memory starts    |
// | ......           |
// | ......           |
// |__________________|

pub const IMMIX_SPACE_ALIGN: ByteSize = (1 << 34); // 16GB
pub const IMMIX_SPACE_LOWBITS_MASK: usize = !(IMMIX_SPACE_ALIGN - 1);

// preallocating 16 GB for immix space
pub const LOG_BYTES_PREALLOC_IMMIX_SPACE: usize = 34;
pub const BYTES_PREALLOC_IMMIX_SPACE: ByteSize = 1 << LOG_BYTES_PREALLOC_IMMIX_SPACE;

// 64KB Immix Block
pub const LOG_BYTES_IN_BLOCK: usize = 16;
pub const BYTES_IN_BLOCK: ByteSize = 1 << LOG_BYTES_IN_BLOCK;

// 256B Immix line
pub const LOG_BYTES_IN_LINE: usize = 8;
pub const BYTES_IN_LINE: ByteSize = (1 << LOG_BYTES_IN_LINE);

// 256K blocks per space
pub const BLOCKS_IN_SPACE: usize = 1 << (LOG_BYTES_PREALLOC_IMMIX_SPACE - LOG_BYTES_IN_BLOCK);
// 64M lines per space
pub const LINES_IN_SPACE: usize = 1 << (LOG_BYTES_PREALLOC_IMMIX_SPACE - LOG_BYTES_IN_LINE);
// 2G words per space
pub const WORDS_IN_SPACE: usize = 1 << (LOG_BYTES_PREALLOC_IMMIX_SPACE - LOG_POINTER_SIZE);
// 256 lines per block
pub const LINES_IN_BLOCK: usize = 1 << (LOG_BYTES_IN_BLOCK - LOG_BYTES_IN_LINE);

// 64KB space metadata (we do not need this much though, but for alignment, we use 64KB)
pub const BYTES_META_SPACE: ByteSize = BYTES_IN_BLOCk;
// 256KB block mark table (1 byte per block)
pub const BYTES_META_BLOCK_MARK_TABLE: ByteSize = BLOCKS_IN_SPACE;
// 64MB line mark table
pub const BYTES_META_LINE_MARK_TABLE: ByteSize = LINES_IN_SPACE;
// 1GB GC byte table
pub const BYTES_META_GC_TABLE: ByteSize = WORDS_IN_SPACE >> 1;
// 1GB TYPE byte table
pub const BYTES_META_TYPE_TABLE: ByteSize = WORDS_IN_SPACE >> 2;

pub const OFFSET_META_BLOCK_MARK_TABLE: ByteOffset = BYTES_META_SPACE as ByteOffset;
pub const OFFSET_META_LINE_MARK_TABLE: ByteOffset =
    OFFSET_META_BLOCK_MARK_TABLE + BYTES_META_BLOCK_MARK_TABLE as ByteOffset;
pub const OFFSET_META_GC_TABLE: ByteOffset =
    OFFSET_META_LINE_MARK_TABLE + BYTES_META_LINE_MARK_TABLE as ByteOffset;
pub const OFFSET_META_TYPE_TABLE: ByteOffset =
    OFFSET_META_META_GC_TABLE + BYTES_META_GC_TABLE as ByteOffset;

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
