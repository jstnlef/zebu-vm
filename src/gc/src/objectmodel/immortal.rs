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

use objectmodel::*;
use utils::*;

pub const IMMORTAL_OBJECT_HEADER_SIZE: ByteSize = 32;

/// We use a 32-bytes header for immortal objects, and the header is
/// put immediately before the object.
#[repr(C, packed)]
pub struct ImmortalObjectHeader {
    pub encode: ObjectEncode,
    pub gc_byte: u8
}
