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

extern crate log;
extern crate simple_logger;
#[macro_use]
extern crate lazy_static;
extern crate rustc_serialize;
extern crate utils;

#[macro_export]
macro_rules! impl_mu_entity {
    ($entity: ty) => {
        impl MuEntity for $entity {
            #[inline(always)]
            fn id(&self) -> MuID {self.hdr.id()}
            #[inline(always)]
            fn name(&self) -> Option<MuName> {self.hdr.name()}
            fn as_entity(&self) -> &MuEntity {
                let ref_ty : &$entity = self;
                ref_ty as &MuEntity
            }
        }
    }
}

macro_rules! select_value {
    ($cond: expr, $res1 : expr, $res2 : expr) => {
        if $cond {
            $res1
        } else {
            $res2
        }
    }
}

#[macro_use]
pub mod ir;
pub mod bundle;
pub mod inst;
pub mod types;
pub mod ir_semantics;
pub mod ptr;
pub mod op;
