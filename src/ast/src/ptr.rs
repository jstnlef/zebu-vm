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

use std::sync::Arc;

/// P<T> is alias type for sharable Mu AST components (such as Value, MuFuncSig, MuType, etc)
// This design is similar to P<T> in rustc compiler.
// However, instead of using Box<T>, we use Arc<T> to encourage sharing
pub type P<T> = Arc<T>;

#[allow(non_snake_case)]
/// Construct a `P<T>` from a `T` value.
pub fn P<T>(value: T) -> P<T> {
    Arc::new(value)
}
