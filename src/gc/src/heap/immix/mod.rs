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

mod immix_space;
mod immix_mutator;

pub use self::immix_space::ImmixSpace;
pub use self::immix_mutator::ImmixMutatorLocal;
pub use self::immix_mutator::ImmixMutatorGlobal;
pub use self::immix_space::LineMarkTable as ImmixLineMarkTable;
pub use self::immix_mutator::MUTATORS;
pub use self::immix_mutator::N_MUTATORS;
pub use self::immix_mutator::CURSOR_OFFSET;
pub use self::immix_mutator::LIMIT_OFFSET;

pub const LOG_BYTES_IN_LINE  : usize = 8;
pub const BYTES_IN_LINE      : usize = (1 << LOG_BYTES_IN_LINE);
pub const LOG_BYTES_IN_BLOCK : usize = 16;
pub const BYTES_IN_BLOCK     : usize = (1 << LOG_BYTES_IN_BLOCK); 
pub const LINES_IN_BLOCK     : usize = (1 << (LOG_BYTES_IN_BLOCK - LOG_BYTES_IN_LINE));

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum LineMark {
    Free,
    Live,
    FreshAlloc,
    ConservLive,
    PrevLive
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BlockMark {
    Usable,
    Full
}
