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

// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The AST pointer
//!
//! Provides `P<T>`, a frozen owned smart pointer, as a replacement for `@T` in
//! the AST.
//!
//! # Motivations and benefits
//!
//! * **Identity**: sharing AST nodes is problematic for the various analysis
//!   passes (e.g. one may be able to bypass the borrow checker with a shared
//!   `ExprAddrOf` node taking a mutable borrow). The only reason `@T` in the
//!   AST hasn't caused issues is because of inefficient folding passes which
//!   would always deduplicate any such shared nodes. Even if the AST were to
//!   switch to an arena, this would still hold, i.e. it couldn't use `&'a T`,
//!   but rather a wrapper like `P<'a, T>`.
//!
//! * **Immutability**: `P<T>` disallows mutating its inner `T`, unlike `Box<T>`
//!   (unless it contains an `Unsafe` interior, but that may be denied later).
//!   This mainly prevents mistakes, but can also enforces a kind of "purity".
//!
//! * **Efficiency**: folding can reuse allocation space for `P<T>` and `Vec<T>`,
//!   the latter even when the input and output types differ (as it would be the
//!   case with arenas or a GADT AST using type parameters to toggle features).
//!
//! * **Maintainability**: `P<T>` provides a fixed interface - `Deref`,
//!   `and_then` and `map` - which can remain fully functional even if the
//!   implementation changes (using a special thread-local heap, for example).
//!   Moreover, a switch to, e.g. `P<'a, T>` would be easy and mostly automated.

//use std::fmt::{self, Display, Debug};
//use std::hash::{Hash, Hasher};
//use std::ops::Deref;
//use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

use std::sync::Arc;

pub type P<T> = Arc<T>;
//pub struct P<T: MuEntity> {
//    ptr: Arc<T>
//}

#[allow(non_snake_case)]
/// Construct a `P<T>` from a `T` value.
pub fn P<T>(value: T) -> P<T> {
//    P {ptr: Arc::new(value)}
    Arc::new(value)
}

//impl<T: MuEntity> Deref for P<T> {
//    type Target = T;
//
//    fn deref<'a>(&'a self) -> &'a T {
//        &*self.ptr
//    }
//}
//
//impl<T: MuEntity> Clone for P<T> {
//    fn clone(&self) -> P<T> {
//        P {ptr: self.ptr.clone()}
//    }
//}
//
//impl<T: MuEntity + PartialEq> PartialEq for P<T> {
//    fn eq(&self, other: &P<T>) -> bool {
//        **self == **other
//    }
//}
//
//impl<T: MuEntity + Eq> Eq for P<T> {}
//
//impl<T: MuEntity + Debug> Debug for P<T> {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        Debug::fmt(&**self, f)
//    }
//}
//impl<T: MuEntity + Display> Display for P<T> {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        Display::fmt(&**self, f)
//    }
//}
//
//impl<T: MuEntity> fmt::Pointer for P<T> {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        fmt::Pointer::fmt(&self.ptr, f)
//    }
//}
//
//impl<T: MuEntity + Hash> Hash for P<T> {
//    fn hash<H: Hasher>(&self, state: &mut H) {
//        (**self).hash(state);
//    }
//}

//impl<T: MuEntity> Encodable for P<T> {
//    fn encode<S: Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
//        s.emit_usize(self.id())
//    }
//}
