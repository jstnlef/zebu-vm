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

mod bitmap;
mod address_bitmap;
mod address_map;
pub mod ptr;
pub mod gctype;
pub mod objectdump;

pub use self::address_bitmap::AddressBitmap;
pub use self::address_map::AddressMap;

pub const SIZE_1KB: ByteSize = 1 << 10;
pub const SIZE_1MB: ByteSize = 1 << 20;
pub const SIZE_1GB: ByteSize = 1 << 30;