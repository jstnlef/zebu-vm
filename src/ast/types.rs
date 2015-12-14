extern crate std;

use std::collections::HashMap;
use std::sync::RwLock;

pub type MuID  = usize;
pub type MuTag = &'static str;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MuType {
    /// int <length>
    Int          (usize),
    /// float
    Float,
    /// double
    Double,
    
    /// ref<T>
    Ref          (Box<MuType>),    // Box is needed for non-recursive enum
    /// iref<T>: internal reference
    IRef         (Box<MuType>),
    /// weakref<T>
    WeakRef      (Box<MuType>),
    
    /// uptr<T>: unsafe pointer
    UPtr         (Box<MuType>),
    
    /// struct<T1 T2 ...>
    Struct       (MuTag),
    
    /// array<T length>
    Array        (Box<MuType>, usize),
    
    /// hybrid<F1 F2 ... V>: a hybrid of fixed length parts and a variable length part
    Hybrid       (Vec<Box<MuType>>, Box<MuType>),
    
    /// void
    Void,
    
    /// threadref
    ThreadRef,
    /// stackref
    StackRef,
    
    /// tagref64: hold a double or an int or an ref<void>
    Tagref64,
    
    /// vector<T length>
    Vector       (Box<MuType>, usize),
    
    /// funcref<@sig>
    FuncRef      (MuFuncSig),
    
    /// ufuncptr<@sig>
    UFuncPtr     (MuFuncSig),
}

lazy_static! {
    /// storing a map from MuTag to StructType_
    static ref STRUCT_TAG_MAP : RwLock<HashMap<MuTag, StructType_>> = RwLock::new(HashMap::new());
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructType_ {
    tys: Vec<Box<MuType>>
}

impl StructType_ {
    pub fn set_tys(&mut self, list: Vec<&MuType>) {
        self.tys.clear();
        self.tys.append(&mut list.into_iter().map(|t| Box::new(t.clone())).collect());
    }
}

impl MuType {
    pub fn int(len: usize) -> MuType {
        MuType::Int(len)
    }
    pub fn float() -> MuType {
        MuType::Float
    }
    pub fn double() -> MuType {
        MuType::Double
    }
    pub fn muref(referent: &MuType) -> MuType {
        MuType::Ref(Box::new(referent.clone()))
    }
    pub fn muref_void() -> MuType {
        MuType::Ref(Box::new(MuType::void()))
    }
    pub fn iref(referent: &MuType) -> MuType {
        MuType::IRef(Box::new(referent.clone()))
    }
    pub fn weakref(referent: &MuType) -> MuType {
        MuType::WeakRef(Box::new(referent.clone()))
    }
    pub fn uptr(referent: &MuType) -> MuType {
        MuType::UPtr(Box::new(referent.clone()))
    }
    pub fn mustruct_empty(tag: MuTag) -> MuType {
        let struct_ty_ = StructType_{tys: vec![]};
        STRUCT_TAG_MAP.write().unwrap().insert(tag, struct_ty_);

        MuType::Struct(tag)
    }
    pub fn mustruct(tag: MuTag, list: Vec<&MuType>) -> MuType {
        let struct_ty_ = StructType_{tys: list.into_iter().map(|t| Box::new(t.clone())).collect()};

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

        MuType::Struct(tag)
    }
    pub fn array(ty: &MuType, len: usize) -> MuType {
        MuType::Array(Box::new(ty.clone()), len)
    }
    pub fn hybrid(fix_tys: Vec<&MuType>, var_ty: &MuType) -> MuType {
        MuType::Hybrid(
            fix_tys.into_iter().map(|t| Box::new(t.clone())).collect(),
            Box::new(var_ty.clone())
        )
    }
    pub fn void() -> MuType {
        MuType::Void
    }
    pub fn threadref() -> MuType {
        MuType::ThreadRef
    }
    pub fn stackref() -> MuType {
        MuType::StackRef
    }
    pub fn tagref64() -> MuType {
        MuType::Tagref64
    }
    pub fn vector(ty: &MuType, len: usize) -> MuType {
        MuType::Vector(Box::new(ty.clone()), len)
    }
    pub fn funcref(sig: MuFuncSig) -> MuType {
        MuType::FuncRef(sig)
    }
    pub fn ufuncptr(sig: MuFuncSig) -> MuType {
        MuType::UFuncPtr(sig)
    }
}

/// is a type floating-point type?
pub fn is_fp(ty: &MuType) -> bool {
    match *ty {
        MuType::Float | MuType::Double => true,
        _ => false
    }
}

/// is a type raw pointer?
pub fn is_ptr(ty: &MuType) -> bool {
    match *ty {
        MuType::UPtr(_) | MuType::UFuncPtr(_) => true,
        _ => false
    }
}

/// is a type scalar type?
pub fn is_scalar(ty: &MuType) -> bool {
    match *ty {
        MuType::Int(_)
        | MuType::Float
        | MuType::Double
        | MuType::Ref(_)
        | MuType::IRef(_)
        | MuType::WeakRef(_)
        | MuType::FuncRef(_)
        | MuType::UFuncPtr(_)
        | MuType::ThreadRef
        | MuType::StackRef
        | MuType::Tagref64
        | MuType::UPtr(_) => true,
        _ => false
    }
}

/// is a type traced by the garbage collector?
/// Note: An aggregated type is traced if any of its part is traced. 
pub fn is_traced(ty: &MuType) -> bool {
    match *ty {
        MuType::Ref(_) => true,
        MuType::IRef(_) => true,
        MuType::WeakRef(_) => true,
        MuType::Array(ref elem_ty, _)
        | MuType::Vector(ref elem_ty, _) => is_traced(elem_ty),
        MuType::ThreadRef
        | MuType::StackRef
        | MuType::Tagref64 => true,
        MuType::Hybrid(ref fix_tys, ref var_ty) => {
            is_traced(var_ty) ||
            fix_tys.into_iter().map(|ty| is_traced(ty))
                .fold(false, |ret, this| ret || this) 
            },
        MuType::Struct(tag) => {
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
        MuType::Int(_) => true,
        MuType::Float => true,
        MuType::Double => true,
        MuType::Void => true,
        MuType::Array(ref elem_ty, _)
        | MuType::Vector(ref elem_ty, _) => is_native_safe(elem_ty),
        MuType::UPtr(_) => true,
        MuType::UFuncPtr(_) => true,
        MuType::Hybrid(ref fix_tys, ref var_ty) => {
            is_native_safe(var_ty) && 
            fix_tys.into_iter().map(|ty| is_native_safe(&ty))
                .fold(true, |ret, this| ret && this)
        },
        MuType::Struct(tag) => {
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MuFuncSig {
    ret_tys : Vec<MuType>,
    args_tys: Vec<MuType>
}

impl MuFuncSig {
    pub fn new(ret_tys: Vec<&MuType>, args_tys: Vec<&MuType>) -> MuFuncSig {
        MuFuncSig {
            ret_tys : ret_tys.into_iter().map(|t| t.clone()).collect(),
            args_tys: args_tys.into_iter().map(|t| t.clone()).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::STRUCT_TAG_MAP;
    
    macro_rules! assert_type (
        ($test:expr, $expect: expr) => (
            assert_eq!(format!("{:?}", $test), $expect)
        )
    );
    
    macro_rules! println_type (
        ($test:expr) => (
            println!("{:?}", $test)
        )  
    );
    
    /// create one of each MuType
    fn create_types() -> Vec<Box<MuType>> {
        let mut types = vec![];
        
        let t0 = MuType::int(8);
        types.push(Box::new(t0));
        
        let t1 = MuType::float();
        types.push(Box::new(t1));
        
        let t2 = MuType::double();
        types.push(Box::new(t2));
        
        let t3 = MuType::muref(&types[0]);
        types.push(Box::new(t3));
        
        let t4 = MuType::iref(&types[0]);
        types.push(Box::new(t4));
        
        let t5 = MuType::weakref(&types[0]);
        types.push(Box::new(t5));
        
        let t6 = MuType::uptr(&types[0]);
        types.push(Box::new(t6));
        
        let t7 = MuType::mustruct("MyStructTag1", vec![&types[0], &types[1]]);
        types.push(Box::new(t7));
        
        let t8 = MuType::array(&types[0], 5);
        types.push(Box::new(t8));
        
        let t9 = MuType::hybrid(vec![&types[7], &types[1]], &types[0]);
        types.push(Box::new(t9));
        
        let t10 = MuType::void();
        types.push(Box::new(t10));
        
        let t11 = MuType::threadref();
        types.push(Box::new(t11));
        
        let t12 = MuType::stackref();
        types.push(Box::new(t12));
        
        let t13 = MuType::tagref64();
        types.push(Box::new(t13));
        
        let t14 = MuType::vector(&types[0], 5);
        types.push(Box::new(t14));
        
        let sig = MuFuncSig::new(vec![&types[10]], vec![&types[0], &types[0]]);
        
        let t15 = MuType::funcref(sig.clone());
        types.push(Box::new(t15));
        
        let t16 = MuType::ufuncptr(sig.clone());
        types.push(Box::new(t16));
        
        types
    }
    
    #[test]
    #[allow(unused_variables)]
    fn test_type_constructors() {
        let types = create_types();
        
        assert_type!(*types[0], "Int(8)");
        assert_type!(*types[1], "Float");
        assert_type!(*types[2], "Double");
        assert_type!(*types[3], "Ref(Int(8))");
        assert_type!(*types[4], "IRef(Int(8))");
        assert_type!(*types[5], "WeakRef(Int(8))");
        assert_type!(*types[6], "UPtr(Int(8))");
        assert_type!(*types[7], "Struct(\"MyStructTag1\")");
        {
            let map = STRUCT_TAG_MAP.read().unwrap();
            let t7_struct_ty = map.get("MyStructTag1").unwrap();
            assert_type!(t7_struct_ty, "StructType_ { tys: [Int(8), Float] }");
        }
        assert_type!(*types[8], "Array(Int(8), 5)");
        assert_type!(*types[9], "Hybrid([Struct(\"MyStructTag1\"), Float], Int(8))");
        assert_type!(*types[10], "Void");
        assert_type!(*types[11], "ThreadRef");
        assert_type!(*types[12], "StackRef");
        assert_type!(*types[13], "Tagref64");
        assert_type!(*types[14], "Vector(Int(8), 5)");
        assert_type!(*types[15], "FuncRef(MuFuncSig { ret_tys: [Void], args_tys: [Int(8), Int(8)] })");
        assert_type!(*types[16], "UFuncPtr(MuFuncSig { ret_tys: [Void], args_tys: [Int(8), Int(8)] })");
    }
    
    #[test]
    fn test_cyclic_struct() {
        // .typedef @cyclic_struct_ty = struct<ref<@cyclic_struct_ty> int<32>>
        let ty = MuType::mustruct_empty("MyStructTag2");
        let ref_ty = MuType::muref(&ty);
        let i32_ty = MuType::int(32);
        
        {
            STRUCT_TAG_MAP.write().unwrap().
                get_mut("MyStructTag2").unwrap().set_tys(vec![&ref_ty, &i32_ty]);
        }
        
        let map = STRUCT_TAG_MAP.read().unwrap();
        let struct_ty = map.get("MyStructTag2").unwrap();
        assert_type!(struct_ty, "StructType_ { tys: [Ref(Struct(\"MyStructTag2\")), Int(32)] }");
    }
    
    #[test]
    fn test_is_traced() {
        let types = create_types();
        
        assert_eq!(is_traced(&types[0]), false);
        assert_eq!(is_traced(&types[1]), false);
        assert_eq!(is_traced(&types[2]), false);
        assert_eq!(is_traced(&types[3]), true);
        assert_eq!(is_traced(&types[4]), true);
        assert_eq!(is_traced(&types[5]), true);
        assert_eq!(is_traced(&types[6]), false);
        assert_eq!(is_traced(&types[7]), false);
        let struct3 = MuType::mustruct("MyStructTag3", vec![&types[3], &types[0]]);
        assert_eq!(is_traced(&struct3), true);
        let struct4 = MuType::mustruct("MyStructTag4", vec![&types[3], &types[4]]);
        assert_eq!(is_traced(&struct4), true);
        assert_eq!(is_traced(&types[8]), false);
        let ref_array = MuType::array(&types[3], 5);
        assert_eq!(is_traced(&ref_array), true);
        assert_eq!(is_traced(&types[9]), false);
        let fix_ref_hybrid = MuType::hybrid(vec![&types[3], &types[0]], &types[0]);
        assert_eq!(is_traced(&fix_ref_hybrid), true);
        let var_ref_hybrid = MuType::hybrid(vec![&types[0], &types[1]], &types[3]);
        assert_eq!(is_traced(&var_ref_hybrid), true);
        assert_eq!(is_traced(&types[10]), false);
        assert_eq!(is_traced(&types[11]), true);
        assert_eq!(is_traced(&types[12]), true);
        assert_eq!(is_traced(&types[13]), true);
        assert_eq!(is_traced(&types[14]), false);
        assert_eq!(is_traced(&types[15]), false);
        assert_eq!(is_traced(&types[16]), false);
    }
    
    #[test]
    fn test_is_native_safe() {
        let types = create_types();    
        
        assert_eq!(is_native_safe(&types[0]), true);
        assert_eq!(is_native_safe(&types[1]), true);
        assert_eq!(is_native_safe(&types[2]), true);
        assert_eq!(is_native_safe(&types[3]), false);
        assert_eq!(is_native_safe(&types[4]), false);
        assert_eq!(is_native_safe(&types[5]), false);
        assert_eq!(is_native_safe(&types[6]), true);
        assert_eq!(is_native_safe(&types[7]), true);
        let struct3 = MuType::mustruct("MyStructTag3", vec![&types[3], &types[0]]);
        assert_eq!(is_native_safe(&struct3), false);
        let struct4 = MuType::mustruct("MyStructTag4", vec![&types[3], &types[4]]);
        assert_eq!(is_native_safe(&struct4), false);
        assert_eq!(is_native_safe(&types[8]), true);
        let ref_array = MuType::array(&types[3], 5);
        assert_eq!(is_native_safe(&ref_array), false);
        assert_eq!(is_native_safe(&types[9]), true);
        let fix_ref_hybrid = MuType::hybrid(vec![&types[3], &types[0]], &types[0]);
        assert_eq!(is_native_safe(&fix_ref_hybrid), false);
        let var_ref_hybrid = MuType::hybrid(vec![&types[0], &types[1]], &types[3]);
        assert_eq!(is_native_safe(&var_ref_hybrid), false);
        assert_eq!(is_native_safe(&types[10]), true);
        assert_eq!(is_native_safe(&types[11]), false);
        assert_eq!(is_native_safe(&types[12]), false);
        assert_eq!(is_native_safe(&types[13]), false);
        assert_eq!(is_native_safe(&types[14]), true);
        assert_eq!(is_native_safe(&types[15]), false);    // funcref is not native safe
                                                          // and not traced either
        assert_eq!(is_native_safe(&types[16]), true);
    }
}