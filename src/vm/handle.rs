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

/// APIHandle represents the opaque handle type that the client uses to
/// communicate with Mu. Handles can refer to values, functions, signatures,
/// etc that client can inspect/query from the VM.
#[derive(Clone)]
pub struct APIHandle {
    pub id: MuID,
    pub v: APIHandleValue
}

/// when we returning an API handle to the client, we create a Box<APIHandle>,
/// then api_impl will turn it into a raw pointer, and pass the pointer the the client.
/// Thus Rust allocates the handle, but will not reclaim it. When the client explicitly
/// deletes a value, we turn the pointer back to a box type, and let Rust drop it.
pub type APIHandleResult = Box<APIHandle>;
/// when client pass a handle (*const APIHandle) to the VM, we treat it as a reference
/// to APIHandle.
pub type APIHandleArg<'a> = &'a APIHandle;

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
    /// (int value, bit length)
    Int(u64, BitSize),
    /// float value
    Float(f32),
    /// double value
    Double(f64),
    /// unsafe pointer (type, address)
    UPtr(P<MuType>, Address), // uptr<T>
    /// unsafe function pointer (type, address)
    UFP(P<MuType>, Address), // ufuncptr<sig>

    // SeqValue
    /// struct value (a vector of field values)
    Struct(Vec<APIHandleValue>),
    /// array value  (a vector of element values)
    Array(Vec<APIHandleValue>),
    /// vector value (a vector of element values)
    Vector(Vec<APIHandleValue>),

    // GenRef
    /// reference value (type, address)
    Ref(P<MuType>, Address),
    /// internal reference value (type, address)
    IRef(P<MuType>, Address),
    /// tagref value (stored as 64-bit integers)
    TagRef64(u64),
    /// function reference (as ID)
    FuncRef(MuID),
    /// Mu thread reference
    ThreadRef,
    /// Mu stack reference
    StackRef,
    /// frame cursor reference
    FCRef,

    // GenRef->IR
    /// Mu bundle
    //  TODO: unused
    Bundle,

    // GenRef->IR->Child
    /// Mu type (as ID)
    Type(MuID),
    /// Mu signature (as ID)
    FuncSig(MuID),
    /// Mu function version (as ID)
    FuncVer(MuID),
    /// basic block
    //  TODO: unused
    BB,
    /// instruction
    //  TODO: unused
    Inst,

    // GenRef->IR->Child->Var->Global
    /// global cell (as ID)
    Global(MuID),
    /// exposed function
    //  TODO: unused
    ExpFunc,

    // GenRef->IR->Child->Var->Local
    /// normal parameter
    //  TODO: unused
    NorParam,
    /// exceptional parameter
    //  TODO: unused
    ExcParam,
    /// instruction result value
    //  TODO: unused
    InstRes
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
            &Int(val, len) => write!(f, "{} as int<{}>", val, len),
            &Float(val) => write!(f, "{}", val),
            &Double(val) => write!(f, "{}", val),
            &UPtr(ref ty, addr) => write!(f, "uptr<{}> to {}", ty, addr),
            &UFP(ref sig, addr) => write!(f, "ufp<{}> to {}", sig, addr),
            &Struct(ref vec) => write!(f, "struct{{{:?}}}", vec),
            &Array(ref vec) => write!(f, "array{{{:?}}}", vec),
            &Vector(ref vec) => write!(f, "vector{{{:?}}}", vec),
            &Ref(ref ty, addr) => write!(f, "ref<{}> to {}", ty, addr),
            &IRef(ref ty, addr) => write!(f, "iref<{}> to {}", ty, addr),
            &TagRef64(val) => write!(f, "tagref64 0x{:x}", val),
            &FuncRef(id) => write!(f, "funcref to #{}", id),
            &ThreadRef => write!(f, "threadref"),
            &StackRef => write!(f, "stackref"),
            &FCRef => write!(f, "framecursorref"),
            &Bundle => write!(f, "IR.bundle"),
            &Type(id) => write!(f, "IR.type to #{}", id),
            &FuncSig(id) => write!(f, "IR.funcsig to #{}", id),
            &FuncVer(id) => write!(f, "IR.funcver to #{}", id),
            &BB => write!(f, "IR.BB"),
            &Inst => write!(f, "IR.inst"),
            &Global(id) => write!(f, "IR.global to #{}", id),
            &ExpFunc => write!(f, "IR.expfunc"),
            &NorParam => write!(f, "IR.norparam"),
            &ExcParam => write!(f, "IR.excparam"),
            &InstRes => write!(f, "IR.instres")
        }
    }
}

impl APIHandleValue {
    /// matches the handle as ref or iref
    pub fn as_ref_or_iref(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::Ref(ref ty, addr) | &APIHandleValue::IRef(ref ty, addr) => {
                (ty.clone(), addr)
            }
            _ => panic!("expected Ref or IRef handle")
        }
    }

    /// matches the handle as ref
    pub fn as_ref(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::Ref(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected Ref handle")
        }
    }

    /// matches the handle as iref
    pub fn as_iref(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::IRef(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected IRef handle")
        }
    }

    /// matches iref/ref/uptr/ufp handles and extracts address
    pub fn as_address(&self) -> Address {
        match self {
            &APIHandleValue::IRef(_, addr)
            | &APIHandleValue::Ref(_, addr)
            | &APIHandleValue::UPtr(_, addr)
            | &APIHandleValue::UFP(_, addr) => addr,
            _ => panic!(
                "expected iref/ref/uptr/ufp which contains a pointer, found {}",
                self
            )
        }
    }

    /// matches the handle as int
    pub fn as_int(&self) -> u64 {
        match self {
            &APIHandleValue::Int(val, _) => val,
            _ => panic!("expected Int handle")
        }
    }

    /// matches the handle as float
    pub fn as_float(&self) -> f32 {
        match self {
            &APIHandleValue::Float(val) => val,
            _ => panic!("expected Float handle")
        }
    }

    /// matches the handle as double
    pub fn as_double(&self) -> f64 {
        match self {
            &APIHandleValue::Double(val) => val,
            _ => panic!("expected Double handle")
        }
    }

    /// matches the handle as unsafe pointer
    pub fn as_uptr(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::UPtr(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected UPtr handle")
        }
    }

    /// matches the handle as unsafe function pointer
    pub fn as_ufp(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::UFP(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected UFP handle")
        }
    }

    /// matches the handle as function reference
    pub fn as_funcref(&self) -> MuID {
        match self {
            &APIHandleValue::FuncRef(id) => id,
            _ => panic!("expected FuncRef")
        }
    }

    /// matches the handle as tag reference's value)
    pub fn as_tr64(&self) -> u64 {
        match self {
            &APIHandleValue::TagRef64(val) => val,
            _ => panic!("expected TagRef64 handle")
        }
    }
}
