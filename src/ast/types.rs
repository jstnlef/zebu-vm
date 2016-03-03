extern crate std;

use ast::ir::*;
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MuType_ {
    /// int <length>
    Int          (usize),
    /// float
    Float,
    /// double
    Double,
    
    /// ref<T>
    Ref          (Arc<MuType_>),    // Box is needed for non-recursive enum
    /// iref<T>: internal reference
    IRef         (Arc<MuType_>),
    /// weakref<T>
    WeakRef      (Arc<MuType_>),
    
    /// uptr<T>: unsafe pointer
    UPtr         (Arc<MuType_>),
    
    /// struct<T1 T2 ...>
    Struct       (MuTag),
    
    /// array<T length>
    Array        (Arc<MuType_>, usize),
    
    /// hybrid<F1 F2 ... V>: a hybrid of fixed length parts and a variable length part
    Hybrid       (Vec<Arc<MuType_>>, Arc<MuType_>),
    
    /// void
    Void,
    
    /// threadref
    ThreadRef,
    /// stackref
    StackRef,
    
    /// tagref64: hold a double or an int or an ref<void>
    Tagref64,
    
    /// vector<T length>
    Vector       (Arc<MuType_>, usize),
    
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
    tys: Vec<Arc<MuType_>>
}

impl StructType_ {
    pub fn set_tys(&mut self, mut list: Vec<Arc<MuType_>>) {
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
    pub fn muref(referent: Arc<MuType_>) -> MuType_ {
        MuType_::Ref(referent)
    }
    pub fn muref_void() -> MuType_ {
        MuType_::Ref(Arc::new(MuType_::void()))
    }
    pub fn iref(referent: Arc<MuType_>) -> MuType_ {
        MuType_::IRef(referent)
    }
    pub fn weakref(referent: Arc<MuType_>) -> MuType_ {
        MuType_::WeakRef(referent)
    }
    pub fn uptr(referent: Arc<MuType_>) -> MuType_ {
        MuType_::UPtr(referent)
    }
    pub fn mustruct_empty(tag: MuTag) -> MuType_ {
        let struct_ty_ = StructType_{tys: vec![]};
        STRUCT_TAG_MAP.write().unwrap().insert(tag, struct_ty_);

        MuType_::Struct(tag)
    }
    pub fn mustruct(tag: MuTag, list: Vec<Arc<MuType_>>) -> MuType_ {
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
    pub fn array(ty: Arc<MuType_>, len: usize) -> MuType_ {
        MuType_::Array(ty, len)
    }
    pub fn hybrid(fix_tys: Vec<Arc<MuType_>>, var_ty: Arc<MuType_>) -> MuType_ {
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
    pub fn vector(ty: Arc<MuType_>, len: usize) -> MuType_ {
        MuType_::Vector(ty, len)
    }
    pub fn funcref(sig: MuFuncSig) -> MuType_ {
        MuType_::FuncRef(sig)
    }
    pub fn ufuncptr(sig: MuFuncSig) -> MuType_ {
        MuType_::UFuncPtr(sig)
    }
}

/// is a type floating-point type?
pub fn is_fp(ty: &MuType_) -> bool {
    match *ty {
        MuType_::Float | MuType_::Double => true,
        _ => false
    }
}

/// is a type raw pointer?
pub fn is_ptr(ty: &MuType_) -> bool {
    match *ty {
        MuType_::UPtr(_) | MuType_::UFuncPtr(_) => true,
        _ => false
    }
}

/// is a type scalar type?
pub fn is_scalar(ty: &MuType_) -> bool {
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
pub fn is_traced(ty: &MuType_) -> bool {
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
pub fn is_native_safe(ty: &MuType_) -> bool {
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MuFuncSig {
    ret_tys : Vec<Arc<MuType_>>,
    args_tys: Vec<Arc<MuType_>>
}

impl MuFuncSig {
    pub fn new(ret_tys: Vec<Arc<MuType_>>, args_tys: Vec<Arc<MuType_>>) -> MuFuncSig {
        MuFuncSig {ret_tys : ret_tys, args_tys: args_tys}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::STRUCT_TAG_MAP;
    use std::sync::Arc;
    
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
    
    /// create one of each MuType_
    fn create_types() -> Vec<Arc<MuType_>> {
        let mut types = vec![];
        
        let t0 = MuType_::int(8);
        types.push(Arc::new(t0));
        
        let t1 = MuType_::float();
        types.push(Arc::new(t1));
        
        let t2 = MuType_::double();
        types.push(Arc::new(t2));
        
        let t3 = MuType_::muref(types[0].clone());
        types.push(Arc::new(t3));
        
        let t4 = MuType_::iref(types[0].clone());
        types.push(Arc::new(t4));
        
        let t5 = MuType_::weakref(types[0].clone());
        types.push(Arc::new(t5));
        
        let t6 = MuType_::uptr(types[0].clone());
        types.push(Arc::new(t6));
        
        let t7 = MuType_::mustruct("MyStructTag1", vec![types[0].clone(), types[1].clone()]);
        types.push(Arc::new(t7));
        
        let t8 = MuType_::array(types[0].clone(), 5);
        types.push(Arc::new(t8));
        
        let t9 = MuType_::hybrid(vec![types[7].clone(), types[1].clone()], types[0].clone());
        types.push(Arc::new(t9));
        
        let t10 = MuType_::void();
        types.push(Arc::new(t10));
        
        let t11 = MuType_::threadref();
        types.push(Arc::new(t11));
        
        let t12 = MuType_::stackref();
        types.push(Arc::new(t12));
        
        let t13 = MuType_::tagref64();
        types.push(Arc::new(t13));
        
        let t14 = MuType_::vector(types[0].clone(), 5);
        types.push(Arc::new(t14));
        
        let sig = MuFuncSig::new(vec![types[10].clone()], vec![types[0].clone(), types[0].clone()]);
        
        let t15 = MuType_::funcref(sig.clone());
        types.push(Arc::new(t15));
        
        let t16 = MuType_::ufuncptr(sig.clone());
        types.push(Arc::new(t16));
        
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
        let ty = Arc::new(MuType_::mustruct_empty("MyStructTag2"));
        let ref_ty = Arc::new(MuType_::muref(ty.clone()));
        let i32_ty = Arc::new(MuType_::int(32));
        
        {
            STRUCT_TAG_MAP.write().unwrap().
                get_mut("MyStructTag2").unwrap().set_tys(vec![ref_ty.clone(), i32_ty.clone()]);
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
        let struct3 = MuType_::mustruct("MyStructTag3", vec![types[3].clone(), types[0].clone()]);
        assert_eq!(is_traced(&struct3), true);
        let struct4 = MuType_::mustruct("MyStructTag4", vec![types[3].clone(), types[4].clone()]);
        assert_eq!(is_traced(&struct4), true);
        assert_eq!(is_traced(&types[8]), false);
        let ref_array = MuType_::array(types[3].clone(), 5);
        assert_eq!(is_traced(&ref_array), true);
        assert_eq!(is_traced(&types[9]), false);
        let fix_ref_hybrid = MuType_::hybrid(vec![types[3].clone(), types[0].clone()], types[0].clone());
        assert_eq!(is_traced(&fix_ref_hybrid), true);
        let var_ref_hybrid = MuType_::hybrid(vec![types[0].clone(), types[1].clone()], types[3].clone());
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
        let struct3 = MuType_::mustruct("MyStructTag3", vec![types[3].clone(), types[0].clone()]);
        assert_eq!(is_native_safe(&struct3), false);
        let struct4 = MuType_::mustruct("MyStructTag4", vec![types[3].clone(), types[4].clone()]);
        assert_eq!(is_native_safe(&struct4), false);
        assert_eq!(is_native_safe(&types[8]), true);
        let ref_array = MuType_::array(types[3].clone(), 5);
        assert_eq!(is_native_safe(&ref_array), false);
        assert_eq!(is_native_safe(&types[9]), true);
        let fix_ref_hybrid = MuType_::hybrid(vec![types[3].clone(), types[0].clone()], types[0].clone());
        assert_eq!(is_native_safe(&fix_ref_hybrid), false);
        let var_ref_hybrid = MuType_::hybrid(vec![types[0].clone(), types[1].clone()], types[3].clone());
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