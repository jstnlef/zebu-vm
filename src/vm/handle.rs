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

use ast::ir::*;
use ast::ptr::*;
use ast::types::*;

use utils::BitSize;
use utils::Address;

use std::fmt;

pub type APIHandleResult = Box<APIHandle>;
pub type APIHandleArg<'a>    = &'a APIHandle;

#[derive(Clone)]
pub struct APIHandle {
    pub id: MuID,
    pub v: APIHandleValue
}

impl fmt::Display for APIHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl fmt::Debug for APIHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Handle#{}=[{:?}]", self.id, self.v)
    }
}

#[derive(Clone)]
pub enum APIHandleValue {
    Int(u64, BitSize),
    Float(f32),
    Double(f64),
    UPtr(P<MuType>, Address),  // uptr<T>
    UFP (P<MuType>, Address),  // ufuncptr<sig>

    // SeqValue
    Struct(Vec<APIHandleValue>),
    Array (Vec<APIHandleValue>),
    Vector(Vec<APIHandleValue>),

    // GenRef
    Ref (P<MuType>, Address),   // referenced type
    IRef(P<MuType>, Address),
    TagRef64(u64),
    FuncRef(MuID),
    ThreadRef,
    StackRef,
    FCRef, // frame cursor ref

    // GenRef->IR
    Bundle,

    // GenRef->IR->Child
    Type(MuID),
    FuncSig(MuID),
    FuncVer(MuID),
    BB,
    Inst,

    // GenRef->IR->Child->Var->Global
    Global(MuID),
    ExpFunc,

    // GenRef->IR->Child->Var->Local
    NorParam,
    ExcParam,
    InstRes,
}

impl fmt::Display for APIHandleValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl fmt::Debug for APIHandleValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::APIHandleValue::*;
        match self {
            &Int(val, len)            => write!(f, "{} as int<{}>", val, len),
            &Float(val)               => write!(f, "{}", val),
            &Double(val)              => write!(f, "{}", val),
            &UPtr(ref ty, addr)           => write!(f, "uptr<{}> to {}", ty, addr),
            &UFP(ref sig, addr)           => write!(f, "ufp<{}> to {}", sig, addr),
            &Struct(ref vec)          => write!(f, "struct{{{:?}}}", vec),
            &Array(ref vec)           => write!(f, "array{{{:?}}}", vec),
            &Vector(ref vec)          => write!(f, "vector{{{:?}}}", vec),
            &Ref(ref ty, addr)        => write!(f, "ref<{}> to {}", ty, addr),
            &IRef(ref ty, addr)       => write!(f, "iref<{}> to {}", ty, addr),
            &TagRef64(val)            => write!(f, "tagref64 0x{:x}", val),
            &FuncRef(id)              => write!(f, "funcref to #{}", id),
            &ThreadRef                => write!(f, "threadref"),
            &StackRef                 => write!(f, "stackref"),
            &FCRef                    => write!(f, "framecursorref"),
            &Bundle                   => write!(f, "IR.bundle"),
            &Type(id)                 => write!(f, "IR.type to #{}", id),
            &FuncSig(id)              => write!(f, "IR.funcsig to #{}", id),
            &FuncVer(id)              => write!(f, "IR.funcver to #{}", id),
            &BB                       => write!(f, "IR.BB"),
            &Inst                     => write!(f, "IR.inst"),
            &Global(id)               => write!(f, "IR.global to #{}", id),
            &ExpFunc                  => write!(f, "IR.expfunc"),
            &NorParam                 => write!(f, "IR.norparam"),
            &ExcParam                 => write!(f, "IR.excparam"),
            &InstRes                  => write!(f, "IR.instres")
        }
    }
}

impl APIHandleValue {
    pub fn as_ref_or_iref(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::Ref(ref ty, addr)
            | &APIHandleValue::IRef(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected Ref or IRef handle")
        }
    }

    pub fn as_ref(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::Ref(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected Ref handle")
        }
    }

    pub fn as_iref(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::IRef(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected IRef handle")
        }
    }

    pub fn as_address(&self) -> Address {
        match self {
            &APIHandleValue::IRef  (_, addr)
            | &APIHandleValue::Ref (_, addr)
            | &APIHandleValue::UPtr(_, addr)
            | &APIHandleValue::UFP (_, addr) => addr,
            _ => panic!("expected iref/ref/uptr/ufp which contains a pointer, found {}", self)
        }
    }

    pub fn as_int(&self) -> u64 {
        match self {
            &APIHandleValue::Int(val, _) => val,
            _ => panic!("expected Int handle")
        }
    }

    pub fn as_float(&self) -> f32 {
        match self {
            &APIHandleValue::Float(val) => val,
            _ => panic!("expected Float handle")
        }
    }

    pub fn as_double(&self) -> f64 {
        match self {
            &APIHandleValue::Double(val) => val,
            _ => panic!("expected Double handle")
        }
    }

    pub fn as_uptr(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::UPtr(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected UPtr handle")
        }
    }

    pub fn as_ufp(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::UFP(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected UFP handle")
        }
    }

    pub fn as_funcref(&self) -> MuID {
        match self {
            &APIHandleValue::FuncRef(id) => id,
            _ => panic!("expected FuncRef")
        }
    }
    
    pub fn as_tr64(&self) -> u64 {
        match self {
            &APIHandleValue::TagRef64(val) => val,
            _ => panic!("expected TagRef64 handle")
        }
    }
}
