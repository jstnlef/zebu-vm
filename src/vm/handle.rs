use ast::ir::*;
use ast::inst::*;
use ast::ptr::*;
use ast::types::*;

use utils::BitSize;
use utils::Address;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct APIHandle {
    pub id: MuID,
    pub v: APIHandleValue
}

#[derive(Clone, Debug)]
pub enum APIHandleValue {
    Int(u64, BitSize),
    Float(f32),
    Double(f64),
    UPtr(Address),
    UFP(Address),

    // SeqValue
    Struct(Vec<APIHandleValue>),
    Array (Vec<APIHandleValue>),
    Vector(Vec<APIHandleValue>),

    // GenRef
    Ref (P<MuType>, Address),   // referenced type
    IRef(P<MuType>, Address),
    TagRef64(u64),
    FuncRef,
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
    Const,
    Global(MuID),
    Func,
    ExpFunc,

    // GenRef->IR->Child->Var->Local
    NorParam,
    ExcParam,
    InstRes,
}

impl APIHandleValue {
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

    pub fn as_int(&self) -> u64 {
        match self {
            &APIHandleValue::Int(val, _) => val,
            _ => panic!("expected Int handle")
        }
    }
}

pub fn store(ord: MemoryOrder, loc: Arc<APIHandle>, val: Arc<APIHandle>) {
    // FIXME: take memory order into consideration

    // get address
    let (_, addr) = loc.v.as_iref();

    // get value and store
    // we will store here (its unsafe)
    unsafe {
        match val.v {
            APIHandleValue::Int(ival, bits) => {
                match bits {
                    8 => addr.store::<u8>(ival as u8),
                    16 => addr.store::<u16>(ival as u16),
                    32 => addr.store::<u32>(ival as u32),
                    64 => addr.store::<u64>(ival),
                    _ => panic!("unimplemented int length")
                }
            },
            APIHandleValue::Float(fval) => addr.store::<f32>(fval),
            APIHandleValue::Double(fval) => addr.store::<f64>(fval),
            APIHandleValue::UPtr(aval) => addr.store::<Address>(aval),
            APIHandleValue::UFP(aval) => addr.store::<Address>(aval),

            APIHandleValue::Struct(_)
            | APIHandleValue::Array(_)
            | APIHandleValue::Vector(_) => panic!("cannot store an aggregated value to an address"),

            APIHandleValue::Ref(_, aval)
            | APIHandleValue::IRef(_, aval) => addr.store::<Address>(aval),

            _ => unimplemented!()
        }
    }
}