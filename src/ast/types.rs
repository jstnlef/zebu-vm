use std::fmt;

use ast::ptr::P;
use ast::ir::*;
use std::collections::HashMap;
use std::sync::RwLock;

pub type MuType = MuType_;

#[derive(Clone, PartialEq, Eq)]
pub enum MuType_ {
    /// int <length>
    Int          (usize),
    /// float
    Float,
    /// double
    Double,
    
    /// ref<T>
    Ref          (P<MuType>),    // Box is needed for non-recursive enum
    /// iref<T>: internal reference
    IRef         (P<MuType>),
    /// weakref<T>
    WeakRef      (P<MuType>),
    
    /// uptr<T>: unsafe pointer
    UPtr         (P<MuType>),
    
    /// struct<T1 T2 ...>
    Struct       (MuTag),
    
    /// array<T length>
    Array        (P<MuType>, usize),
    
    /// hybrid<F1 F2 ... V>: a hybrid of fixed length parts and a variable length part
    Hybrid       (Vec<P<MuType>>, P<MuType>),
    
    /// void
    Void,
    
    /// threadref
    ThreadRef,
    /// stackref
    StackRef,
    
    /// tagref64: hold a double or an int or an ref<void>
    Tagref64,
    
    /// vector<T length>
    Vector       (P<MuType>, usize),
    
    /// funcref<@sig>
    FuncRef      (P<MuFuncSig>),
    
    /// ufuncptr<@sig>
    UFuncPtr     (P<MuFuncSig>),
}

impl fmt::Debug for MuType_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &MuType_::Int(n)                          => write!(f, "int<{}>", n),
            &MuType_::Float                           => write!(f, "float"),
            &MuType_::Double                          => write!(f, "double"),
            &MuType_::Ref(ref ty)                     => write!(f, "ref<{:?}>", ty),
            &MuType_::IRef(ref ty)                    => write!(f, "iref<{:?}>", ty),
            &MuType_::WeakRef(ref ty)                 => write!(f, "weakref<{:?}>", ty),
            &MuType_::UPtr(ref ty)                    => write!(f, "uptr<{:?}>", ty),
            &MuType_::Array(ref ty, size)             => write!(f, "array<{:?} {:?}>", ty, size),
            &MuType_::Hybrid(ref fix_tys, ref var_ty) => write!(f, "hybrid<{:?} {:?}>", fix_tys, var_ty), 
            &MuType_::Void                            => write!(f, "void"),
            &MuType_::ThreadRef                       => write!(f, "threadref"),
            &MuType_::StackRef                        => write!(f, "stackref"),
            &MuType_::Tagref64                        => write!(f, "tagref64"),
            &MuType_::Vector(ref ty, size)            => write!(f, "vector<{:?} {:?}>", ty, size),
            &MuType_::FuncRef(ref sig)                => write!(f, "funcref<{:?}>", sig),
            &MuType_::UFuncPtr(ref sig)               => write!(f, "ufuncref<{:?}>", sig),
            &MuType_::Struct(tag)                     => write!(f, "{}(struct)", tag)
        }
    }
}

lazy_static! {
    /// storing a map from MuTag to StructType_
    pub static ref STRUCT_TAG_MAP : RwLock<HashMap<MuTag, StructType_>> = RwLock::new(HashMap::new());
}

#[derive(Clone, PartialEq, Eq)]
pub struct StructType_ {
    tys: Vec<P<MuType_>>
}

impl fmt::Debug for StructType_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "struct<").unwrap();
        for i in 0..self.tys.len() {
            let ty = &self.tys[i];
            write!(f, "{:?}", ty).unwrap();
            if i != self.tys.len() - 1 {
                write!(f, " ").unwrap();
            }
        }
        write!(f, ">")
    }    
}

impl StructType_ {
    pub fn set_tys(&mut self, mut list: Vec<P<MuType>>) {
        self.tys.clear();
        self.tys.append(&mut list);
    }
}

impl MuType_ {
    pub fn int(len: usize) -> MuType_ {
        MuType_::Int(len)
    }
    pub fn float() -> MuType_ {
        MuType_::Float
    }
    pub fn double() -> MuType_ {
        MuType_::Double
    }
    pub fn muref(referent: P<MuType_>) -> MuType_ {
        MuType_::Ref(referent)
    }
    pub fn muref_void() -> MuType_ {
        MuType_::Ref(P(MuType_::void()))
    }
    pub fn iref(referent: P<MuType_>) -> MuType_ {
        MuType_::IRef(referent)
    }
    pub fn weakref(referent: P<MuType_>) -> MuType_ {
        MuType_::WeakRef(referent)
    }
    pub fn uptr(referent: P<MuType_>) -> MuType_ {
        MuType_::UPtr(referent)
    }
    pub fn mustruct_empty(tag: MuTag) -> MuType_ {
        let struct_ty_ = StructType_{tys: vec![]};
        STRUCT_TAG_MAP.write().unwrap().insert(tag, struct_ty_);

        MuType_::Struct(tag)
    }
    pub fn mustruct(tag: MuTag, list: Vec<P<MuType_>>) -> MuType_ {
        let struct_ty_ = StructType_{tys: list};

        // if there is an attempt to use a same tag for different struct,
        // we panic
        match STRUCT_TAG_MAP.read().unwrap().get(tag) {
            Some(old_struct_ty_) => {
                if struct_ty_ != *old_struct_ty_ {
                    panic!(format!(
                            "trying to insert {:?} as {}, while the old struct is defined as {:?}",
                            struct_ty_, tag, old_struct_ty_))
                }
            },
            None => {}
        }
        // otherwise, store the tag
        STRUCT_TAG_MAP.write().unwrap().insert(tag, struct_ty_);

        MuType_::Struct(tag)
    }
    pub fn array(ty: P<MuType_>, len: usize) -> MuType_ {
        MuType_::Array(ty, len)
    }
    pub fn hybrid(fix_tys: Vec<P<MuType_>>, var_ty: P<MuType_>) -> MuType_ {
        MuType_::Hybrid(fix_tys, var_ty)
    }
    pub fn void() -> MuType_ {
        MuType_::Void
    }
    pub fn threadref() -> MuType_ {
        MuType_::ThreadRef
    }
    pub fn stackref() -> MuType_ {
        MuType_::StackRef
    }
    pub fn tagref64() -> MuType_ {
        MuType_::Tagref64
    }
    pub fn vector(ty: P<MuType_>, len: usize) -> MuType_ {
        MuType_::Vector(ty, len)
    }
    pub fn funcref(sig: P<MuFuncSig>) -> MuType_ {
        MuType_::FuncRef(sig)
    }
    pub fn ufuncptr(sig: P<MuFuncSig>) -> MuType_ {
        MuType_::UFuncPtr(sig)
    }
}

/// is a type floating-point type?
pub fn is_fp(ty: &MuType) -> bool {
    match *ty {
        MuType_::Float | MuType_::Double => true,
        _ => false
    }
}

/// is a type raw pointer?
pub fn is_ptr(ty: &MuType) -> bool {
    match *ty {
        MuType_::UPtr(_) | MuType_::UFuncPtr(_) => true,
        _ => false
    }
}

/// is a type scalar type?
pub fn is_scalar(ty: &MuType) -> bool {
    match *ty {
        MuType_::Int(_)
        | MuType_::Float
        | MuType_::Double
        | MuType_::Ref(_)
        | MuType_::IRef(_)
        | MuType_::WeakRef(_)
        | MuType_::FuncRef(_)
        | MuType_::UFuncPtr(_)
        | MuType_::ThreadRef
        | MuType_::StackRef
        | MuType_::Tagref64
        | MuType_::UPtr(_) => true,
        _ => false
    }
}

/// is a type traced by the garbage collector?
/// Note: An aggregated type is traced if any of its part is traced. 
pub fn is_traced(ty: &MuType) -> bool {
    match *ty {
        MuType_::Ref(_) => true,
        MuType_::IRef(_) => true,
        MuType_::WeakRef(_) => true,
        MuType_::Array(ref elem_ty, _)
        | MuType_::Vector(ref elem_ty, _) => is_traced(elem_ty),
        MuType_::ThreadRef
        | MuType_::StackRef
        | MuType_::Tagref64 => true,
        MuType_::Hybrid(ref fix_tys, ref var_ty) => {
            is_traced(var_ty) ||
            fix_tys.into_iter().map(|ty| is_traced(ty))
                .fold(false, |ret, this| ret || this) 
            },
        MuType_::Struct(tag) => {
            let map = STRUCT_TAG_MAP.read().unwrap();
            let struct_ty = map.get(tag).unwrap();
            let ref field_tys = struct_ty.tys;
            
            field_tys.into_iter().map(|ty| is_traced(&ty))
                .fold(false, |ret, this| ret || this)
        },
        _ => false
    }
}

/// is a type native safe?
/// Note: An aggregated type is native safe if all of its parts are native safe.
pub fn is_native_safe(ty: &MuType) -> bool {
    match *ty {
        MuType_::Int(_) => true,
        MuType_::Float => true,
        MuType_::Double => true,
        MuType_::Void => true,
        MuType_::Array(ref elem_ty, _)
        | MuType_::Vector(ref elem_ty, _) => is_native_safe(elem_ty),
        MuType_::UPtr(_) => true,
        MuType_::UFuncPtr(_) => true,
        MuType_::Hybrid(ref fix_tys, ref var_ty) => {
            is_native_safe(var_ty) && 
            fix_tys.into_iter().map(|ty| is_native_safe(&ty))
                .fold(true, |ret, this| ret && this)
        },
        MuType_::Struct(tag) => {
            let map = STRUCT_TAG_MAP.read().unwrap();
            let struct_ty = map.get(tag).unwrap();
            let ref field_tys = struct_ty.tys;
            
            field_tys.into_iter().map(|ty| is_native_safe(&ty))
                .fold(true, |ret, this| ret && this)
        },
        _ => false
    }
}

macro_rules! is_type (
    ($e:expr, $p:pat) => (
        match $e {
            $p => true,
            _ => false
        }
    )
);

#[derive(Clone, PartialEq, Eq)]
pub struct MuFuncSig {
    pub ret_tys : Vec<P<MuType>>,
    pub arg_tys: Vec<P<MuType>>
}

impl fmt::Debug for MuFuncSig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} -> {:?}", self.ret_tys, self.arg_tys)
    }    
}